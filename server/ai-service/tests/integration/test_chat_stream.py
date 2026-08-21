import json
from collections.abc import AsyncIterator
from typing import Any
from uuid import UUID, uuid4

import pytest
from fastapi.testclient import TestClient
from sse_starlette.sse import AppStatus

from ai_service.llm.client import AssistantTurn, TokenChunk
from ai_service.main import create_app


@pytest.fixture(autouse=True)
def _reset_sse_app_status() -> None:
    """sse-starlette caches a per-loop Event; reset between tests."""
    AppStatus.should_exit_event = None
    yield
    AppStatus.should_exit_event = None


class FakeLLM:
    """Drop-in async-iterating stub that mimics streaming chat."""

    model = "gpt-4o-mini-fake"

    def __init__(self, tokens: list[str]) -> None:
        self._tokens = tokens

    async def chat_stream(
        self,
        messages: list[dict[str, str]],
        *,
        max_tokens: int,
        tools: list[dict[str, Any]] | None = None,
        tool_choice: str | None = None,
    ) -> AsyncIterator[TokenChunk | AssistantTurn]:
        text_parts: list[str] = []
        for t in self._tokens:
            text_parts.append(t)
            yield TokenChunk(text=t)
        yield AssistantTurn(text="".join(text_parts), tool_calls=[], finish_reason="stop")


def _make_app_with_fake_llm(tokens: list[str]) -> TestClient:
    app = create_app()
    client = TestClient(app)
    client.__enter__()
    app.state.llm_client = FakeLLM(tokens)  # type: ignore[assignment]
    return client


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


@pytest.fixture
def fake_client() -> TestClient:
    client = _make_app_with_fake_llm(["Привет", ", ", "как ", "дела?"])
    yield client
    client.__exit__(None, None, None)


def _headers(user_id: UUID) -> dict[str, str]:
    return {"X-User-Id": str(user_id), "X-User-Role": "customer"}


def test_chat_stream_emits_tokens_and_done(fake_client: TestClient) -> None:
    user_id = uuid4()
    create = fake_client.post("/api/stylist/conversations", headers=_headers(user_id))
    assert create.status_code == 201
    conv_id = create.json()["conversation_id"]

    with fake_client.stream(
        "POST",
        "/api/stylist/chat",
        json={"conversation_id": conv_id, "message": "Здравствуй"},
        headers=_headers(user_id),
    ) as resp:
        assert resp.status_code == 200
        body = b"".join(resp.iter_bytes())

    events = _parse_sse(body)
    kinds = [e[0] for e in events]
    assert kinds.count("token") == 4
    assert "done" in kinds
    assert "error" not in kinds
    text = "".join(e[1]["text"] for e in events if e[0] == "token")
    assert text == "Привет, как дела?"


def test_chat_rejects_unknown_conversation(fake_client: TestClient) -> None:
    user_id = uuid4()
    with fake_client.stream(
        "POST",
        "/api/stylist/chat",
        json={"conversation_id": str(uuid4()), "message": "hi"},
        headers=_headers(user_id),
    ) as resp:
        assert resp.status_code == 200
        events = _parse_sse(b"".join(resp.iter_bytes()))
    kinds = [e[0] for e in events]
    assert "error" in kinds
    error_event = next(e for e in events if e[0] == "error")
    assert error_event[1]["code"] == "bad_request"
    assert kinds[-1] == "done"


def test_delete_conversation_idempotent(fake_client: TestClient) -> None:
    user_id = uuid4()
    create = fake_client.post("/api/stylist/conversations", headers=_headers(user_id))
    conv_id = create.json()["conversation_id"]
    r1 = fake_client.delete(f"/api/stylist/conversations/{conv_id}", headers=_headers(user_id))
    r2 = fake_client.delete(f"/api/stylist/conversations/{conv_id}", headers=_headers(user_id))
    assert r1.status_code == 204
    assert r2.status_code == 204
