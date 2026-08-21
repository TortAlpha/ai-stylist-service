from datetime import timedelta
from uuid import uuid4

import pytest

from ai_service.domain.conversation import ConversationStore


@pytest.mark.asyncio
async def test_create_and_get() -> None:
    store = ConversationStore(ttl_minutes=30, max_messages=10)
    user = uuid4()
    conv = await store.create(user)
    fetched = await store.get(conv.id, user)
    assert fetched is not None
    assert fetched.id == conv.id


@pytest.mark.asyncio
async def test_get_rejects_other_user() -> None:
    store = ConversationStore(ttl_minutes=30, max_messages=10)
    a, b = uuid4(), uuid4()
    conv = await store.create(a)
    assert await store.get(conv.id, b) is None


@pytest.mark.asyncio
async def test_ttl_drops_stale() -> None:
    store = ConversationStore(ttl_minutes=30, max_messages=10)
    user = uuid4()
    conv = await store.create(user)
    conv.last_active = conv.last_active - timedelta(hours=1)
    assert await store.get(conv.id, user) is None
    assert store.size() == 0


@pytest.mark.asyncio
async def test_trim_to_cap() -> None:
    store = ConversationStore(ttl_minutes=30, max_messages=3)
    user = uuid4()
    conv = await store.create(user)
    for i in range(5):
        conv.append("user", f"m{i}")
    dropped = conv.trim_to_cap(store.max_messages)
    assert len(dropped) == 2
    assert len(conv.messages) == 3
    assert conv.messages[0].content == "m2"


@pytest.mark.asyncio
async def test_delete_idempotent() -> None:
    store = ConversationStore(ttl_minutes=30, max_messages=10)
    user = uuid4()
    conv = await store.create(user)
    assert await store.delete(conv.id, user) is True
    assert await store.delete(conv.id, user) is False
