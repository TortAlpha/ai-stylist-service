import asyncio
from collections import defaultdict
from datetime import UTC, datetime, timedelta
from typing import Literal
from uuid import UUID, uuid4

from pydantic import BaseModel, Field

from ..metrics import active_conversations

Role = Literal["system", "user", "assistant", "tool"]


def _utcnow() -> datetime:
    return datetime.now(UTC)


def _set_active_gauge(value: int) -> None:
    active_conversations.set(value)


class Message(BaseModel):
    role: Role
    content: str
    tool_call_id: str | None = None
    pruned: bool = False
    created_at: datetime = Field(default_factory=_utcnow)


class Conversation(BaseModel):
    id: UUID = Field(default_factory=uuid4)
    user_id: UUID
    messages: list[Message] = Field(default_factory=list)
    last_product_ids: list[UUID] = Field(default_factory=list)
    created_at: datetime = Field(default_factory=_utcnow)
    last_active: datetime = Field(default_factory=_utcnow)

    def append(self, role: Role, content: str) -> Message:
        msg = Message(role=role, content=content)
        self.messages.append(msg)
        self.last_active = _utcnow()
        return msg

    def trim_to_cap(self, cap: int) -> list[Message]:
        if cap <= 0 or len(self.messages) <= cap:
            return []
        drop_count = len(self.messages) - cap
        dropped = self.messages[:drop_count]
        self.messages = self.messages[drop_count:]
        return dropped


class ConversationStore:
    def __init__(self, *, ttl_minutes: int, max_messages: int) -> None:
        self._ttl = timedelta(minutes=ttl_minutes)
        self._max_messages = max_messages
        self._store: dict[UUID, Conversation] = {}
        self._locks: defaultdict[UUID, asyncio.Lock] = defaultdict(asyncio.Lock)
        self._cleanup_task: asyncio.Task[None] | None = None
        self._stopped = asyncio.Event()

    @property
    def max_messages(self) -> int:
        return self._max_messages

    async def create(self, user_id: UUID) -> Conversation:
        conv = Conversation(user_id=user_id)
        self._store[conv.id] = conv
        _set_active_gauge(len(self._store))
        return conv

    async def get(self, conversation_id: UUID, user_id: UUID) -> Conversation | None:
        conv = self._store.get(conversation_id)
        if conv is None or conv.user_id != user_id:
            return None
        if _utcnow() - conv.last_active > self._ttl:
            self._store.pop(conversation_id, None)
            self._locks.pop(conversation_id, None)
            _set_active_gauge(len(self._store))
            return None
        return conv

    async def delete(self, conversation_id: UUID, user_id: UUID) -> bool:
        conv = self._store.get(conversation_id)
        if conv is None or conv.user_id != user_id:
            return False
        self._store.pop(conversation_id, None)
        self._locks.pop(conversation_id, None)
        _set_active_gauge(len(self._store))
        return True

    def lock(self, conversation_id: UUID) -> asyncio.Lock:
        return self._locks[conversation_id]

    def size(self) -> int:
        return len(self._store)

    async def _cleanup_loop(self, interval_seconds: float) -> None:
        while not self._stopped.is_set():
            try:
                await asyncio.wait_for(self._stopped.wait(), timeout=interval_seconds)
            except TimeoutError:
                self._sweep_expired()
            else:
                return

    def _sweep_expired(self) -> None:
        now = _utcnow()
        expired = [cid for cid, c in self._store.items() if now - c.last_active > self._ttl]
        for cid in expired:
            self._store.pop(cid, None)
            self._locks.pop(cid, None)
        if expired:
            _set_active_gauge(len(self._store))

    async def start(self, interval_seconds: float = 60.0) -> None:
        if self._cleanup_task is None:
            self._stopped.clear()
            self._cleanup_task = asyncio.create_task(self._cleanup_loop(interval_seconds))

    async def stop(self) -> None:
        self._stopped.set()
        if self._cleanup_task is not None:
            self._cleanup_task.cancel()
            try:
                await self._cleanup_task
            except (asyncio.CancelledError, Exception):
                pass
            self._cleanup_task = None
