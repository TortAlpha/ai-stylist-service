import json
from uuid import UUID, uuid4

import pytest

from ai_service.domain.errors import UpstreamUnavailable
from ai_service.embeddings.counter import QueryEmbedCounter
from ai_service.llm.tools import ToolContext, dispatch_tool
from ai_service.product.schemas import (
    FilterOptions,
    ProductDetails,
    ScoreBreakdown,
    SearchFilters,
    SemanticSearchFilters,
    SemanticSearchItem,
    SemanticSearchResponse,
)


class StubEmbedder:
    dim = 1024

    def __init__(self, *, fail: bool = False) -> None:
        self.fail = fail
        self.calls: list[str] = []

    async def embed_text(self, text: str) -> list[float]:
        self.calls.append(text)
        if self.fail:
            raise UpstreamUnavailable("simulated cohere outage")
        return [0.0] * self.dim


class StubProduct:
    def __init__(
        self,
        *,
        items: list[SemanticSearchItem] | None = None,
        detail: ProductDetails | None = None,
    ) -> None:
        self.items = items or []
        self.detail = detail
        self.last_vector: list[float] | None = None
        self.last_top_k: int | None = None
        self.last_filters: SemanticSearchFilters | None = None

    async def get_products(self, _: SearchFilters, **__) -> list:  # pragma: no cover
        raise NotImplementedError

    async def get_filter_options(self, **_) -> FilterOptions:  # pragma: no cover
        return FilterOptions()

    async def get_product(self, _id: UUID, **__) -> ProductDetails:
        if self.detail is None:
            raise UpstreamUnavailable("no detail configured")
        return self.detail

    async def semantic_search(
        self,
        *,
        vector: list[float],
        top_k: int,
        filters: SemanticSearchFilters,
    ) -> SemanticSearchResponse:
        self.last_vector = vector
        self.last_top_k = top_k
        self.last_filters = filters
        return SemanticSearchResponse(items=self.items)


@pytest.mark.asyncio
async def test_semantic_search_happy_path() -> None:
    pid = uuid4()
    items = [
        SemanticSearchItem(
            id=pid,
            score=0.81,
            score_breakdown=ScoreBreakdown(text=0.7, image=0.92, best_image_idx=2),
            name="Vintage Moncler",
            brand="Moncler",
            category="outerwear",
            price=14500,
        )
    ]
    product = StubProduct(items=items)
    embedder = StubEmbedder()
    counter = QueryEmbedCounter(daily_cap=10)
    ctx = ToolContext(product=product, embedder=embedder, query_counter=counter)

    result = await dispatch_tool(
        "semantic_search",
        json.dumps({"query": "тёплое из 90-х", "top_k": 5}),
        ctx=ctx,
    )
    assert "error" not in result.payload
    assert result.product_ids == [str(pid)]
    first = result.payload["items"][0]
    assert first["score_breakdown"]["text"] == 0.7
    assert first["score_breakdown"]["image"] == 0.92
    assert first["score_breakdown"]["best_image_idx"] == 2
    assert embedder.calls == ["тёплое из 90-х"]
    assert product.last_top_k == 5
    assert await counter.used() == 1


@pytest.mark.asyncio
async def test_semantic_search_no_embedder() -> None:
    ctx = ToolContext(
        product=StubProduct(),
        embedder=None,
        query_counter=QueryEmbedCounter(daily_cap=10),
    )
    result = await dispatch_tool("semantic_search", json.dumps({"query": "x"}), ctx=ctx)
    assert result.payload["error"] == "embed_disabled"


@pytest.mark.asyncio
async def test_semantic_search_cap_hit() -> None:
    embedder = StubEmbedder()
    counter = QueryEmbedCounter(daily_cap=0)
    ctx = ToolContext(product=StubProduct(), embedder=embedder, query_counter=counter)
    result = await dispatch_tool("semantic_search", json.dumps({"query": "x"}), ctx=ctx)
    assert result.payload["error"] == "embed_cap"
    assert embedder.calls == []  # cap blocks before embed


@pytest.mark.asyncio
async def test_semantic_search_validation() -> None:
    ctx = ToolContext(
        product=StubProduct(),
        embedder=StubEmbedder(),
        query_counter=QueryEmbedCounter(daily_cap=10),
    )
    result = await dispatch_tool("semantic_search", "{not-json", ctx=ctx)
    assert result.payload["error"] == "validation"


@pytest.mark.asyncio
async def test_get_product_details_happy() -> None:
    pid = uuid4()
    detail = ProductDetails(
        id=pid, name="Test", brand="Brand", category="bags", price=4200, image_count=3
    )
    ctx = ToolContext(product=StubProduct(detail=detail))
    result = await dispatch_tool(
        "get_product_details",
        json.dumps({"id": str(pid)}),
        ctx=ctx,
    )
    assert result.product_ids == [str(pid)]
    assert result.payload["name"] == "Test"
    assert result.payload["image_count"] == 3
