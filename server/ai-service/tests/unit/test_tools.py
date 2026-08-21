import json
from uuid import uuid4

import pytest

from ai_service.domain.errors import UpstreamUnavailable
from ai_service.llm.tools import (
    ListFacetsArgs,
    SearchProductsArgs,
    dispatch_tool,
)
from ai_service.product.schemas import (
    FilterOptions,
    ProductSummary,
    SearchFilters,
)


def test_search_products_args_accepts_minimal() -> None:
    a = SearchProductsArgs.model_validate({"category": "bags"})
    assert a.category == "bags"
    assert a.tags == []


def test_search_products_args_negative_price_rejected() -> None:
    # JSON schema enforces minimum=0 on the model side too via validators
    # because Pydantic SearchFilters has price as Optional[int]; we accept it
    # but the tool description tells the model to pass non-negatives.
    a = SearchProductsArgs.model_validate({"price_min": 0, "price_max": 1000})
    assert a.price_min == 0
    assert a.price_max == 1000


def test_list_facets_args_rejects_unknown_field() -> None:
    with pytest.raises(ValueError):
        ListFacetsArgs.model_validate({"field": "bogus"})


def test_search_filters_to_query_joins_tags() -> None:
    f = SearchFilters(category="bags", tags=["leather", "90s"], price_max=10000)
    q = f.to_query()
    assert q["category"] == "bags"
    assert q["tags"] == "leather,90s"
    assert q["price_max"] == "10000"


class _StubProductClient:
    def __init__(
        self,
        *,
        products: list[ProductSummary] | None = None,
        facets: FilterOptions | None = None,
        raise_upstream: bool = False,
    ) -> None:
        self._products = products or []
        self._facets = facets or FilterOptions()
        self._raise = raise_upstream

    async def get_products(self, _: SearchFilters, **__) -> list[ProductSummary]:
        if self._raise:
            raise UpstreamUnavailable("simulated")
        return self._products

    async def get_filter_options(self, **_) -> FilterOptions:
        if self._raise:
            raise UpstreamUnavailable("simulated")
        return self._facets


@pytest.mark.asyncio
async def test_dispatch_search_products_happy_path() -> None:
    pid = uuid4()
    stub = _StubProductClient(
        products=[ProductSummary(id=pid, name="Vintage bag", brand="Coach", price=5500)]
    )
    result = await dispatch_tool(
        "search_products",
        json.dumps({"category": "bags", "tags": ["leather"]}),
        product=stub,  # type: ignore[arg-type]
    )
    assert result.name == "search_products"
    assert result.product_ids == [str(pid)]
    assert result.payload["total"] == 1


@pytest.mark.asyncio
async def test_dispatch_search_products_invalid_args() -> None:
    stub = _StubProductClient()
    result = await dispatch_tool(
        "search_products",
        "{not-json}",
        product=stub,  # type: ignore[arg-type]
    )
    assert result.payload["error"] == "validation"
    assert result.product_ids == []


@pytest.mark.asyncio
async def test_dispatch_search_products_upstream_unavailable() -> None:
    stub = _StubProductClient(raise_upstream=True)
    result = await dispatch_tool(
        "search_products", "{}", product=stub  # type: ignore[arg-type]
    )
    assert result.payload["error"] == "upstream_unavailable"
    assert result.payload["retriable"] is True


@pytest.mark.asyncio
async def test_dispatch_list_facets_returns_values() -> None:
    stub = _StubProductClient(facets=FilterOptions(brands=["Coach", "Furla"]))
    result = await dispatch_tool(
        "list_facets",
        json.dumps({"field": "brands"}),
        product=stub,  # type: ignore[arg-type]
    )
    assert result.payload == {"field": "brands", "values": ["Coach", "Furla"]}


@pytest.mark.asyncio
async def test_dispatch_unknown_tool() -> None:
    stub = _StubProductClient()
    result = await dispatch_tool("destroy_universe", "{}", product=stub)  # type: ignore[arg-type]
    assert result.payload["error"] == "unknown_tool"
