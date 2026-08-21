import json
from collections.abc import AsyncIterator
from typing import Any
from uuid import UUID, uuid4

import pytest
from fastapi.testclient import TestClient
from sse_starlette.sse import AppStatus

from ai_service.llm.client import AssistantTurn, TokenChunk, ToolCall
from ai_service.main import create_app
from ai_service.product.schemas import (
    FilterOptions,
    ProductSummary,
    SearchFilters,
)


@pytest.fixture(autouse=True)
def _reset_sse_app_status() -> None:
    AppStatus.should_exit_event = None
    yield
    AppStatus.should_exit_event = None


class StubProductClient:
    """In-process stand-in for ProductClient — avoids HTTP entirely."""

    def __init__(
        self,
        *,
        products: list[ProductSummary] | None = None,
        facets: FilterOptions | None = None,
    ) -> None:
        self.products = products or []
        self.facets = facets or FilterOptions()
        self.last_filters: SearchFilters | None = None

    async def get_products(self, filters: SearchFilters, **_) -> list[ProductSummary]:
        self.last_filters = filters
        return self.products

    async def get_product(self, _id, **__):  # pragma: no cover
        raise NotImplementedError

    async def get_filter_options(self, **_) -> FilterOptions:
        return self.facets


class ScriptedLLM:
    """Replays a list of scripted turns to drive the loop deterministically."""

    model = "scripted"

    def __init__(self, turns: list[AssistantTurn]) -> None:
        self._turns = list(turns)
        self.calls = 0
        self.last_tools: list[dict[str, Any]] | None = None
        self.last_tool_choice: str | None = None

    async def chat_stream(
        self,
        messages: list[dict[str, str]],
        *,
        max_tokens: int,
        tools: list[dict[str, Any]] | None = None,
        tool_choice: str | None = None,
    ) -> AsyncIterator[TokenChunk | AssistantTurn]:
        self.calls += 1
        self.last_tools = tools
        self.last_tool_choice = tool_choice
        if not self._turns:
            yield AssistantTurn(text="", tool_calls=[], finish_reason="stop")
            return
        turn = self._turns.pop(0)
        if turn.text:
            yield TokenChunk(text=turn.text)
        yield turn


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


def _bootstrap(
    turns: list[AssistantTurn],
    products: list[ProductSummary],
) -> tuple[TestClient, ScriptedLLM, StubProductClient]:
    app = create_app()
    client = TestClient(app)
    client.__enter__()
    llm = ScriptedLLM(turns)
    pc = StubProductClient(products=products)
    app.state.llm_client = llm  # type: ignore[assignment]
    app.state.product_client = pc  # type: ignore[assignment]
    return client, llm, pc


def test_tool_use_search_products_emits_event_products() -> None:
    pid = uuid4()
    products = [ProductSummary(id=pid, name="Vintage Coach bag", brand="Coach", price=5500)]
    turns = [
        AssistantTurn(
            text="",
            tool_calls=[
                ToolCall(
                    id="call_1",
                    name="search_products",
                    arguments=json.dumps(
                        {"category": "bags", "tags": ["leather"], "price_max": 10000}
                    ),
                )
            ],
            finish_reason="tool_calls",
        ),
        AssistantTurn(
            text="Нашёл подходящие сумки.",
            tool_calls=[],
            finish_reason="stop",
        ),
    ]
    client, llm, pc = _bootstrap(turns, products)
    try:
        user_id = uuid4()
        conv_id = client.post("/api/stylist/conversations", headers=_headers(user_id)).json()[
            "conversation_id"
        ]
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "Кожаные сумки до 10к"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "products" in kinds
    products_event = next(e for e in events if e[0] == "products")
    assert products_event[1]["ids"] == [str(pid)]
    assert "error" not in kinds
    assert kinds[-1] == "done"
    text = "".join(e[1]["text"] for e in events if e[0] == "token")
    assert "Нашёл" in text
    assert pc.last_filters is not None
    assert pc.last_filters.category == "bags"
    assert pc.last_filters.price_max == 10000


def test_tool_use_cap_forces_final_answer() -> None:
    # Always returns a tool_call → loop must hit cap and produce a final reply.
    looping_turn = AssistantTurn(
        text="",
        tool_calls=[
            ToolCall(id="c", name="search_products", arguments="{}"),
        ],
        finish_reason="tool_calls",
    )
    # 5 loop turns; on cap the loop calls again with tool_choice="none".
    turns: list[AssistantTurn] = [looping_turn] * 4 + [
        AssistantTurn(text="Извини, не получилось подобрать.", tool_calls=[], finish_reason="stop")
    ]
    client, llm, _ = _bootstrap(turns, products=[])
    try:
        user_id = uuid4()
        conv_id = client.post("/api/stylist/conversations", headers=_headers(user_id)).json()[
            "conversation_id"
        ]
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "уйми этот цикл"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" not in kinds
    assert kinds[-1] == "done"
    # Final call must have been made with tool_choice="none".
    assert llm.last_tool_choice == "none"
