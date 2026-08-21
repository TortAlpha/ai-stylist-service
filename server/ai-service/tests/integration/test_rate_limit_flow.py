import json
from collections.abc import AsyncIterator
from typing import Any
from uuid import UUID, uuid4

import pytest
from fastapi.testclient import TestClient
from sse_starlette.sse import AppStatus

from ai_service.embeddings.counter import QueryEmbedCounter
from ai_service.llm.client import AssistantTurn, TokenChunk
from ai_service.main import create_app
from ai_service.ratelimit import RateLimiter


@pytest.fixture(autouse=True)
def _reset_sse_app_status() -> None:
    AppStatus.should_exit_event = None
    yield
    AppStatus.should_exit_event = None


class _SpyLLM:
    """Records whether anyone called chat_stream; never returns content."""

    model = "spy"

    def __init__(self) -> None:
        self.calls = 0

    async def chat_stream(
        self,
        messages: list[dict[str, str]],
        *,
        max_tokens: int,
        tools: list[dict[str, Any]] | None = None,
        tool_choice: str | None = None,
    ) -> AsyncIterator[TokenChunk | AssistantTurn]:
        self.calls += 1
        yield AssistantTurn(text="hi", tool_calls=[], finish_reason="stop")


def _parse_sse(raw: bytes) -> list[tuple[str, dict]]:
    events: list[tuple[str, dict]] = []
    current_event: str | None = None
    data_buf: list[str] = []
    for line in raw.decode("utf-8").splitlines():
        if line.startswith("event:"):
            current_event = line[len("event:") :].strip()
        elif line.startswith("data:"):
            data_buf.append(line[len("data:") :].strip())
        elif line == "":
            if current_event is not None:
                payload = "\n".join(data_buf) or "{}"
                try:
                    parsed = json.loads(payload)
                except json.JSONDecodeError:
                    parsed = {"_raw": payload}
                events.append((current_event, parsed))
            current_event, data_buf = None, []
    return events


def _headers(user_id: UUID) -> dict[str, str]:
    return {"X-User-Id": str(user_id), "X-User-Role": "customer"}


def test_rate_limited_second_message_short_circuits_without_llm() -> None:
    app = create_app()
    client = TestClient(app)
    client.__enter__()
    try:
        spy = _SpyLLM()
        app.state.llm_client = spy  # type: ignore[assignment]
        # mpm=1 → second message in the same minute is rejected without
        # touching the LLM.
        app.state.rate_limiter = RateLimiter(messages_per_min=1, tokens_per_day=10_000)
        app.state.query_counter = QueryEmbedCounter(daily_cap=10)

        user_id = uuid4()
        conv_id = client.post(
            "/api/stylist/conversations", headers=_headers(user_id)
        ).json()["conversation_id"]
        # 1st: ok
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "one"},
            headers=_headers(user_id),
        ) as resp:
            list(resp.iter_bytes())  # drain to completion
        assert spy.calls == 1
        # 2nd: rate-limited, no LLM call
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "two"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" in kinds
    err = next(e for e in events if e[0] == "error")
    assert err[1]["code"] == "rate_limited"
    assert kinds[-1] == "done"
    assert spy.calls == 1  # critical: no second OpenAI call


def test_normal_flow_records_tokens() -> None:
    app = create_app()
    client = TestClient(app)
    client.__enter__()
    try:
        rl = RateLimiter(messages_per_min=10, tokens_per_day=10_000)
        app.state.rate_limiter = rl

        class TokensLLM:
            model = "fake"

            async def chat_stream(
                self,
                messages,
                *,
                max_tokens,
                tools=None,
                tool_choice=None,
            ):
                turn = AssistantTurn(
                    text="hi",
                    tool_calls=[],
                    finish_reason="stop",
                    prompt_tokens=10,
                    completion_tokens=5,
                    total_tokens=15,
                )
                yield TokenChunk(text="hi")
                yield turn

        app.state.llm_client = TokensLLM()  # type: ignore[assignment]
        app.state.query_counter = QueryEmbedCounter(daily_cap=10)

        user_id = uuid4()
        conv_id = client.post(
            "/api/stylist/conversations", headers=_headers(user_id)
        ).json()["conversation_id"]
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "test"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))

        # Run rate-limiter check from outside the request to read state.
        import asyncio

        used = asyncio.run(rl.tokens_used_today(user_id))
        assert used == 15
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" not in kinds
    assert kinds[-1] == "done"
