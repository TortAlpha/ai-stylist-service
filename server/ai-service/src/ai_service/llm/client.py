import asyncio
import random
import time
from collections.abc import AsyncIterator
from dataclasses import dataclass, field
from typing import Any, Protocol

import structlog
from openai import APIConnectionError, APITimeoutError, AsyncOpenAI, RateLimitError
from openai._exceptions import APIStatusError

from ..circuit_breaker import CircuitBreaker, CircuitOpen
from ..domain.errors import RateLimited, UpstreamUnavailable
from ..metrics import (
    circuit_breaker_rejections_total,
    openai_request_duration_seconds,
    openai_tokens_total,
    record_breaker_state,
)
from .budget import ChatMessage

log = structlog.get_logger(__name__)


@dataclass(slots=True)
class TokenChunk:
    text: str


@dataclass(slots=True)
class ToolCall:
    """A single tool call produced by the assistant in a streaming response."""

    id: str
    name: str
    arguments: str  # raw JSON string, validated by the caller


@dataclass(slots=True)
class AssistantTurn:
    """Aggregated streaming response from one chat.completions call."""

    text: str = ""
    tool_calls: list[ToolCall] = field(default_factory=list)
    finish_reason: str | None = None
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0

    @property
    def has_tool_calls(self) -> bool:
        return bool(self.tool_calls)


class ChatStream(Protocol):
    def __aiter__(self) -> AsyncIterator[TokenChunk]: ...


class LLMClient:
    def __init__(
        self,
        *,
        api_key: str,
        model: str,
        timeout_seconds: float,
        max_retries: int = 3,
        langsmith_enabled: bool = False,
        breaker: CircuitBreaker | None = None,
    ) -> None:
        self._breaker = breaker
        raw_client = AsyncOpenAI(api_key=api_key, timeout=timeout_seconds)
        if langsmith_enabled:
            try:
                from langsmith.wrappers import wrap_openai

                self._client = wrap_openai(raw_client)
                log.info("llm.langsmith_wrapped", model=model)
            except Exception as exc:  # pragma: no cover — degrade gracefully
                log.warning("llm.langsmith_wrap_failed", error=str(exc))
                self._client = raw_client
        else:
            self._client = raw_client
        self._model = model
        self._max_retries = max_retries

    @property
    def model(self) -> str:
        return self._model

    async def _mark_failure(self) -> None:
        if self._breaker is None:
            return
        await self._breaker.record_failure()
        record_breaker_state("openai", self._breaker.state.value)

    async def chat_stream(
        self,
        messages: list[ChatMessage],
        *,
        max_tokens: int,
        tools: list[dict[str, Any]] | None = None,
        tool_choice: str | None = None,
    ) -> AsyncIterator[TokenChunk | AssistantTurn]:
        """Stream tokens, then yield a final AssistantTurn summary.

        Token chunks come out as `TokenChunk`. The last item is always an
        `AssistantTurn` that aggregates the full text and any tool_calls.
        Callers can route on isinstance to drive the tool-use loop.
        """
        if self._breaker is not None:
            try:
                await self._breaker.check()
            except CircuitOpen:
                circuit_breaker_rejections_total.labels(name="openai").inc()
                record_breaker_state("openai", self._breaker.state.value)
                raise UpstreamUnavailable("openai breaker open") from None
        last_exc: Exception | None = None
        for attempt in range(1, self._max_retries + 1):
            started = time.monotonic()
            try:
                kwargs: dict[str, Any] = {
                    "model": self._model,
                    "messages": messages,
                    "stream": True,
                    "max_tokens": max_tokens,
                    "stream_options": {"include_usage": True},
                }
                if tools:
                    kwargs["tools"] = tools
                if tool_choice is not None:
                    kwargs["tool_choice"] = tool_choice
                stream = await self._client.chat.completions.create(**kwargs)

                turn = AssistantTurn()
                tool_buffers: dict[int, ToolCall] = {}

                async for chunk in stream:
                    usage = getattr(chunk, "usage", None)
                    if usage is not None:
                        turn.prompt_tokens = int(getattr(usage, "prompt_tokens", 0) or 0)
                        turn.completion_tokens = int(getattr(usage, "completion_tokens", 0) or 0)
                        turn.total_tokens = int(getattr(usage, "total_tokens", 0) or 0)
                    if not chunk.choices:
                        continue
                    choice = chunk.choices[0]
                    delta = choice.delta
                    if choice.finish_reason:
                        turn.finish_reason = choice.finish_reason

                    text = getattr(delta, "content", None)
                    if text:
                        turn.text += text
                        yield TokenChunk(text=text)

                    tc_list = getattr(delta, "tool_calls", None) or []
                    for tc in tc_list:
                        idx = getattr(tc, "index", 0) or 0
                        existing = tool_buffers.get(idx)
                        if existing is None:
                            existing = ToolCall(
                                id=tc.id or "",
                                name=(tc.function.name if tc.function else "") or "",
                                arguments=(tc.function.arguments if tc.function else "") or "",
                            )
                            tool_buffers[idx] = existing
                        else:
                            if tc.id:
                                existing.id = tc.id
                            if tc.function and tc.function.name:
                                existing.name = tc.function.name
                            if tc.function and tc.function.arguments:
                                existing.arguments += tc.function.arguments

                turn.tool_calls = [tool_buffers[i] for i in sorted(tool_buffers)]
                openai_request_duration_seconds.labels(model=self._model, kind="stream").observe(
                    time.monotonic() - started
                )
                if turn.prompt_tokens:
                    openai_tokens_total.labels(model=self._model, kind="prompt").inc(
                        turn.prompt_tokens
                    )
                if turn.completion_tokens:
                    openai_tokens_total.labels(model=self._model, kind="completion").inc(
                        turn.completion_tokens
                    )
                if turn.total_tokens:
                    openai_tokens_total.labels(model=self._model, kind="total").inc(
                        turn.total_tokens
                    )
                if self._breaker is not None:
                    await self._breaker.record_success()
                    record_breaker_state("openai", self._breaker.state.value)
                yield turn
                return
            except RateLimitError as exc:
                last_exc = exc
                if attempt >= self._max_retries:
                    log.warning("openai.rate_limited", attempt=attempt)
                    await self._mark_failure()
                    raise RateLimited("openai rate limit") from exc
            except (APITimeoutError, APIConnectionError) as exc:
                last_exc = exc
                if attempt >= self._max_retries:
                    log.warning("openai.unavailable", attempt=attempt, error=str(exc))
                    await self._mark_failure()
                    raise UpstreamUnavailable("openai unavailable") from exc
            except APIStatusError as exc:
                last_exc = exc
                if 500 <= exc.status_code < 600 and attempt < self._max_retries:
                    pass
                elif exc.status_code == 429:
                    await self._mark_failure()
                    raise RateLimited("openai rate limit") from exc
                else:
                    log.warning("openai.api_error", status=exc.status_code)
                    await self._mark_failure()
                    raise UpstreamUnavailable(f"openai status {exc.status_code}") from exc
            await asyncio.sleep(min(2 ** (attempt - 1), 8) + random.uniform(0, 0.2))
        if self._breaker is not None:
            await self._breaker.record_failure()
            record_breaker_state("openai", self._breaker.state.value)
        if last_exc is not None:
            raise UpstreamUnavailable("openai unavailable after retries") from last_exc
