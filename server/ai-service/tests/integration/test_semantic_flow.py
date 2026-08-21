import json
from collections.abc import AsyncIterator
from typing import Any
from uuid import UUID, uuid4

import pytest
from fastapi.testclient import TestClient
from sse_starlette.sse import AppStatus

from ai_service.embeddings.counter import QueryEmbedCounter
from ai_service.llm.client import AssistantTurn, TokenChunk, ToolCall
from ai_service.main import create_app
from ai_service.product.schemas import (
    FilterOptions,
    ProductDetails,
    ScoreBreakdown,
    SearchFilters,
    SemanticSearchFilters,
    SemanticSearchItem,
    SemanticSearchResponse,
)


@pytest.fixture(autouse=True)
def _reset_sse_app_status() -> None:
    AppStatus.should_exit_event = None
    yield
    AppStatus.should_exit_event = None


class StubProduct:
    """In-process stand-in for ProductClient covering semantic-search path."""

    def __init__(
        self,
        *,
        semantic_items: list[SemanticSearchItem] | None = None,
        detail: ProductDetails | None = None,
    ) -> None:
        self.semantic_items = semantic_items or []
        self.detail = detail
        self.last_semantic_filters: SemanticSearchFilters | None = None
        self.last_semantic_top_k: int | None = None

    async def get_products(self, _: SearchFilters, **__):  # pragma: no cover
        return []

    async def get_filter_options(self, **_) -> FilterOptions:  # pragma: no cover
        return FilterOptions()

    async def get_product(self, _id: UUID, **__) -> ProductDetails:
        if self.detail is None:
            raise RuntimeError("no detail configured")
        return self.detail

    async def semantic_search(
        self, *, vector: list[float], top_k: int, filters: SemanticSearchFilters
    ) -> SemanticSearchResponse:
        self.last_semantic_filters = filters
        self.last_semantic_top_k = top_k
        return SemanticSearchResponse(items=self.semantic_items)


class StubEmbedder:
    dim = 1024

    def __init__(self) -> None:
        self.calls: list[str] = []

    async def embed_text(self, text: str) -> list[float]:
        self.calls.append(text)
        return [0.0] * self.dim


class ScriptedLLM:
    model = "scripted"

    def __init__(self, turns: list[AssistantTurn]) -> None:
        self._turns = list(turns)
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
    *,
    product: StubProduct,
    embedder: StubEmbedder | None,
) -> TestClient:
    app = create_app()
    client = TestClient(app)
    client.__enter__()
    app.state.llm_client = ScriptedLLM(turns)  # type: ignore[assignment]
    app.state.product_client = product  # type: ignore[assignment]
    app.state.embedder = embedder
    app.state.query_counter = QueryEmbedCounter(daily_cap=100)
    return client


def test_semantic_search_emits_products_and_score_breakdown_in_tool_result() -> None:
    pid = uuid4()
    items = [
        SemanticSearchItem(
            id=pid,
            score=0.83,
            score_breakdown=ScoreBreakdown(text=0.66, image=0.95, best_image_idx=1),
            name="Шерстяной кардиган Benetton",
            brand="Benetton",
            category="knitwear",
            price=4500,
        )
    ]
    turns = [
        AssistantTurn(
            text="",
            tool_calls=[
                ToolCall(
                    id="call_sem_1",
                    name="semantic_search",
                    arguments=json.dumps({"query": "тёплое из 90-х", "top_k": 5}),
                )
            ],
            finish_reason="tool_calls",
        ),
        AssistantTurn(text="Подошло по виду.", tool_calls=[], finish_reason="stop"),
    ]
    product = StubProduct(semantic_items=items)
    embedder = StubEmbedder()
    client = _bootstrap(turns, product=product, embedder=embedder)
    try:
        user_id = uuid4()
        conv_id = client.post("/api/stylist/conversations", headers=_headers(user_id)).json()[
            "conversation_id"
        ]
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "что-то в духе 90-х, тёплое"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" not in kinds
    assert kinds[-1] == "done"
    products_event = next(e for e in events if e[0] == "products")
    assert products_event[1]["ids"] == [str(pid)]
    assert embedder.calls == ["тёплое из 90-х"]
    assert product.last_semantic_top_k == 5


def test_semantic_search_falls_back_when_embedder_disabled() -> None:
    turns = [
        AssistantTurn(
            text="",
            tool_calls=[
                ToolCall(
                    id="call_sem_2",
                    name="semantic_search",
                    arguments=json.dumps({"query": "тёплое"}),
                )
            ],
            finish_reason="tool_calls",
        ),
        AssistantTurn(
            text="Семантический поиск временно недоступен, попробую обычные фильтры.",
            tool_calls=[],
            finish_reason="stop",
        ),
    ]
    client = _bootstrap(turns, product=StubProduct(), embedder=None)
    try:
        user_id = uuid4()
        conv_id = client.post("/api/stylist/conversations", headers=_headers(user_id)).json()[
            "conversation_id"
        ]
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "тёплое"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" not in kinds
    assert kinds[-1] == "done"
    # No products event when tool returned error.
    assert "products" not in kinds
    text = "".join(e[1]["text"] for e in events if e[0] == "token")
    assert "недоступен" in text or "фильтры" in text


def test_drill_in_uses_get_product_details() -> None:
    pid = uuid4()
    detail = ProductDetails(
        id=pid,
        name="Пуховик Moncler 1996",
        brand="Moncler",
        category="outerwear",
        price=14500,
        description="хаки, размер M",
        size="M",
        condition="excellent",
        image_count=4,
    )
    turns = [
        AssistantTurn(
            text="",
            tool_calls=[
                ToolCall(
                    id="call_detail",
                    name="get_product_details",
                    arguments=json.dumps({"id": str(pid)}),
                )
            ],
            finish_reason="tool_calls",
        ),
        AssistantTurn(
            text="Пуховик Moncler из 1996, размер M, хаки.",
            tool_calls=[],
            finish_reason="stop",
        ),
    ]
    product = StubProduct(detail=detail)
    client = _bootstrap(turns, product=product, embedder=StubEmbedder())
    try:
        user_id = uuid4()
        conv_id = client.post("/api/stylist/conversations", headers=_headers(user_id)).json()[
            "conversation_id"
        ]
        # Seed last_product_ids so the drill-in id can be retrieved naturally
        # via the system prompt — we still ask the LLM to call directly.
        with client.stream(
            "POST",
            "/api/stylist/chat",
            json={"conversation_id": conv_id, "message": "расскажи подробнее"},
            headers=_headers(user_id),
        ) as resp:
            events = _parse_sse(b"".join(resp.iter_bytes()))
    finally:
        client.__exit__(None, None, None)

    kinds = [e[0] for e in events]
    assert "error" not in kinds
    assert kinds[-1] == "done"
    products_event = next(e for e in events if e[0] == "products")
    assert products_event[1]["ids"] == [str(pid)]
    text = "".join(e[1]["text"] for e in events if e[0] == "token")
    assert "Moncler" in text
