import hashlib
import json
import time
from collections.abc import AsyncIterator
from contextlib import nullcontext
from typing import cast
from uuid import UUID

import structlog
from fastapi import APIRouter, Depends, Request
from pydantic import BaseModel, Field
from sse_starlette.sse import EventSourceResponse

from ..auth import User, current_user
from ..config import Settings
from ..deps import (
    get_embedder,
    get_llm_client,
    get_product_client,
    get_query_counter,
    get_settings_dep,
    get_store,
)
from ..domain.conversation import Conversation, ConversationStore, Message
from ..domain.errors import AIServiceError, BadRequest, ErrorCode
from ..embeddings.counter import QueryEmbedCounter
from ..embeddings.multimodal_client import MultimodalEmbedder
from ..llm.budget import ChatMessage
from ..llm.client import LLMClient
from ..llm.loop import LoopOutcome, initial_history, run_loop
from ..llm.prompts import system_prompt_v2
from ..llm.tools import ToolContext
from ..metrics import chat_duration_seconds, chat_messages_total, rate_limit_hits_total
from ..product.client import ProductClient
from ..ratelimit import RateLimiter

log = structlog.get_logger(__name__)


def _hash_user(user_id: UUID) -> str:
    return hashlib.sha256(user_id.bytes).hexdigest()[:16]


def _rate_limiter(request: Request) -> RateLimiter:
    return request.app.state.rate_limiter  # type: ignore[no-any-return]

router = APIRouter(prefix="/api/stylist", tags=["stylist"])

_user_dep = Depends(current_user)
_store_dep = Depends(get_store)
_llm_dep = Depends(get_llm_client)
_settings_dep = Depends(get_settings_dep)
_product_dep = Depends(get_product_client)
_embedder_dep = Depends(get_embedder)
_counter_dep = Depends(get_query_counter)


class ChatRequest(BaseModel):
    conversation_id: UUID
    message: str = Field(min_length=1, max_length=4000)


def _trace_chat(
    *,
    enabled: bool,
    conv_id: UUID,
    user_id: UUID,
    user_text: str,
):
    if not enabled:
        return nullcontext(None)
    try:
        from langsmith.run_helpers import trace

        return trace(
            name="stylist_chat",
            run_type="chain",
            inputs={"message": user_text},
            metadata={
                "conversation_id": str(conv_id),
                "user_id": str(user_id),
            },
        )
    except Exception as exc:  # pragma: no cover — degrade gracefully
        log.warning("chat.langsmith_trace_failed", error=str(exc))
        return nullcontext(None)


def _to_chat_message(msg: Message) -> ChatMessage:
    return cast(ChatMessage, {"role": msg.role, "content": msg.content})


def _sse_event(event: str, data: dict[str, object] | None = None) -> dict[str, str]:
    return {"event": event, "data": json.dumps(data or {}, ensure_ascii=False)}


async def _stream_chat(
    *,
    conv: Conversation,
    store: ConversationStore,
    llm: LLMClient,
    product: ProductClient,
    embedder: MultimodalEmbedder | None,
    counter: QueryEmbedCounter,
    rate_limiter: RateLimiter,
    user_text: str,
    user_role: str,
    settings: Settings,
) -> AsyncIterator[dict[str, str]]:
    async with store.lock(conv.id):
        conv.append("user", user_text)
        conv.trim_to_cap(store.max_messages)

        history = [_to_chat_message(m) for m in conv.messages]
        last_ids = [str(p) for p in conv.last_product_ids]
        messages = initial_history(
            system_prompt_v2(user_role, last_product_ids=last_ids or None),
            history,
        )

        tool_ctx = ToolContext(
            product=product,
            embedder=embedder,
            query_counter=counter,
            as_user=(conv.user_id, user_role),
        )

        trace_ctx = _trace_chat(
            enabled=settings.langsmith_enabled,
            conv_id=conv.id,
            user_id=conv.user_id,
            user_text=user_text,
        )
        final_text = ""
        product_ids: list[str] = []
        with trace_ctx as run:
            try:
                async for item in run_loop(
                    llm=llm,
                    tool_ctx=tool_ctx,
                    base_messages=messages,
                    max_tool_iterations=settings.max_tool_iterations,
                    history_budget=settings.history_token_budget,
                    max_tokens_per_reply=settings.max_tokens_per_reply,
                ):
                    if isinstance(item, LoopOutcome):
                        final_text = item.text
                        product_ids = item.product_ids
                        if item.total_tokens > 0:
                            await rate_limiter.record_tokens(conv.user_id, item.total_tokens)
                        if run is not None:
                            run.end(
                                outputs={
                                    "text": final_text,
                                    "product_ids": product_ids,
                                    "iterations": item.iterations,
                                    "hit_cap": item.hit_cap,
                                    "total_tokens": item.total_tokens,
                                }
                            )
                        break
                    kind, payload = item
                    if kind == "token":
                        yield _sse_event("token", {"text": payload})
                    elif kind == "products":
                        yield _sse_event("products", {"ids": list(payload)})
            except AIServiceError as exc:
                log.warning(
                    "chat.upstream_error",
                    conversation_id=str(conv.id),
                    code=exc.code.value,
                )
                if run is not None:
                    run.end(outputs={"error": exc.code.value})
                yield _sse_event("error", {"code": exc.code.value, "message": exc.message})
                yield _sse_event("done")
                return
            except Exception as exc:
                log.exception(
                    "chat.internal_error", conversation_id=str(conv.id), error=str(exc)
                )
                if run is not None:
                    run.end(outputs={"error": "internal"})
                yield _sse_event(
                    "error",
                    {"code": ErrorCode.INTERNAL.value, "message": "internal error"},
                )
                yield _sse_event("done")
                return

        if final_text:
            conv.append("assistant", final_text)
        if product_ids:
            conv.last_product_ids = [UUID(pid) for pid in product_ids]

    yield _sse_event("done")


@router.post("/chat")
async def chat(
    request: Request,
    body: ChatRequest,
    user: User = _user_dep,
    store: ConversationStore = _store_dep,
    llm: LLMClient = _llm_dep,
    product: ProductClient = _product_dep,
    embedder: MultimodalEmbedder | None = _embedder_dep,
    counter: QueryEmbedCounter = _counter_dep,
    settings: Settings = _settings_dep,
) -> EventSourceResponse:
    rate_limiter: RateLimiter = _rate_limiter(request)
    user_hash = _hash_user(user.id)

    async def event_stream() -> AsyncIterator[dict[str, str]]:
        started = time.monotonic()
        result_label = "ok"
        emitted_error = False

        async def emit_error(code: ErrorCode, message: str) -> AsyncIterator[dict[str, str]]:
            nonlocal result_label, emitted_error
            emitted_error = True
            result_label = code.value
            yield _sse_event("error", {"code": code.value, "message": message})
            yield _sse_event("done")

        log.info(
            "chat.started",
            conversation_id=str(body.conversation_id),
            user_id_hash=user_hash,
        )

        # Cheap pre-flight: never burn an OpenAI call for a rate-limited user.
        decision = await rate_limiter.check_message(user.id)
        if not decision.allowed:
            reason = decision.reason.value if decision.reason else "unknown"
            rate_limit_hits_total.labels(reason=reason).inc()
            log.info("chat.rate_limited", user_id_hash=user_hash, reason=reason)
            async for ev in emit_error(ErrorCode.RATE_LIMITED, f"rate limit: {reason}"):
                yield ev
            chat_messages_total.labels(result=result_label).inc()
            chat_duration_seconds.observe(time.monotonic() - started)
            return

        try:
            conv = await store.get(body.conversation_id, user.id)
            if conv is None:
                raise BadRequest("conversation not found")
        except AIServiceError as exc:
            async for ev in emit_error(exc.code, exc.message):
                yield ev
            chat_messages_total.labels(result=result_label).inc()
            chat_duration_seconds.observe(time.monotonic() - started)
            return

        try:
            async for ev in _stream_chat(
                conv=conv,
                store=store,
                llm=llm,
                product=product,
                embedder=embedder,
                counter=counter,
                rate_limiter=rate_limiter,
                user_text=body.message,
                user_role=user.role,
                settings=settings,
            ):
                if await request.is_disconnected():
                    log.info(
                        "chat.client_disconnected",
                        conversation_id=str(body.conversation_id),
                        user_id_hash=user_hash,
                    )
                    return
                if ev.get("event") == "error" and not emitted_error:
                    try:
                        result_label = json.loads(ev.get("data") or "{}").get("code") or "internal"
                    except json.JSONDecodeError:
                        result_label = "internal"
                    emitted_error = True
                yield ev
        except Exception as exc:  # safety net
            log.exception("chat.fatal", error=str(exc))
            async for ev in emit_error(ErrorCode.INTERNAL, "internal error"):
                yield ev
        finally:
            chat_messages_total.labels(result=result_label).inc()
            chat_duration_seconds.observe(time.monotonic() - started)
            log.info(
                "chat.finished",
                conversation_id=str(body.conversation_id),
                user_id_hash=user_hash,
                result=result_label,
            )

    return EventSourceResponse(event_stream(), ping=15)
