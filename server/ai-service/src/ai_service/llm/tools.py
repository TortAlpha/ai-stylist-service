"""Tool definitions for the OpenAI function-calling protocol.

Each tool has:
- a JSON schema (sent to the model in `tools=[...]`)
- a Pydantic args model (validates the arguments returned by the model)
- a `dispatch` coroutine that calls product-service and returns a JSON-
  serialisable dict shoved back into the conversation as the tool result.

Tool errors are surfaced to the model as `{"error": ..., "...": ...}` so
it can re-plan instead of the chat crashing.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any
from uuid import UUID

import structlog
from pydantic import BaseModel, Field, ValidationError, field_validator

from ..domain.errors import UpstreamUnavailable
from ..embeddings.counter import QueryEmbedCounter
from ..embeddings.multimodal_client import MultimodalEmbedder
from ..product.client import ProductClient
from ..product.schemas import SearchFilters, SemanticSearchFilters

log = structlog.get_logger(__name__)


FACET_FIELDS = ("tags", "brands", "categories", "sizes", "conditions")


SEARCH_PRODUCTS_SCHEMA: dict[str, Any] = {
    "type": "function",
    "function": {
        "name": "search_products",
        "description": (
            "Точечный поиск товаров по фильтрам (бренд, категория, теги, цена, размер, "
            "состояние). Используй, когда у пользователя конкретные критерии. "
            "Если значение не задано пользователем — не передавай поле."
        ),
        "parameters": {
            "type": "object",
            "additionalProperties": False,
            "properties": {
                "category": {"type": "string"},
                "brand": {"type": "string"},
                "tags": {"type": "array", "items": {"type": "string"}},
                "price_min": {"type": "integer", "minimum": 0},
                "price_max": {"type": "integer", "minimum": 0},
                "size": {"type": "string"},
                "condition": {"type": "string"},
            },
        },
    },
}


LIST_FACETS_SCHEMA: dict[str, Any] = {
    "type": "function",
    "function": {
        "name": "list_facets",
        "description": (
            "Список реальных значений фасета (брендов, категорий, тегов, размеров, "
            "состояний). Зови, если не уверен, что значение существует, до того как "
            "фильтровать по нему."
        ),
        "parameters": {
            "type": "object",
            "additionalProperties": False,
            "required": ["field"],
            "properties": {
                "field": {
                    "type": "string",
                    "enum": list(FACET_FIELDS),
                },
            },
        },
    },
}


SEMANTIC_SEARCH_SCHEMA: dict[str, Any] = {
    "type": "function",
    "function": {
        "name": "semantic_search",
        "description": (
            "Поиск по смыслу запроса (для размытых описаний: «в стиле 90-х», "
            "«что-то тёплое и уютное», «с леопардовым принтом»). Возвращает топ-K "
            "товаров со score_breakdown — это объясняет, подошёл ли товар по "
            "описанию (text) или по виду (image)."
        ),
        "parameters": {
            "type": "object",
            "additionalProperties": False,
            "required": ["query"],
            "properties": {
                "query": {"type": "string", "minLength": 1},
                "top_k": {"type": "integer", "minimum": 1, "maximum": 20, "default": 10},
                "filters": {
                    "type": "object",
                    "additionalProperties": False,
                    "properties": {
                        "category": {"type": "string"},
                        "brand": {"type": "string"},
                        "tags": {"type": "array", "items": {"type": "string"}},
                        "size": {"type": "string"},
                        "condition": {"type": "string"},
                        "price_min": {"type": "integer", "minimum": 0},
                        "price_max": {"type": "integer", "minimum": 0},
                    },
                },
            },
        },
    },
}


GET_PRODUCT_DETAILS_SCHEMA: dict[str, Any] = {
    "type": "function",
    "function": {
        "name": "get_product_details",
        "description": (
            "Полная карточка одного товара по id. Используй для drill-in "
            "(«расскажи подробнее про вторую») — id бери из last_product_ids, "
            "которые подставлены в системный контекст."
        ),
        "parameters": {
            "type": "object",
            "additionalProperties": False,
            "required": ["id"],
            "properties": {
                "id": {"type": "string", "format": "uuid"},
            },
        },
    },
}


TOOL_SCHEMAS: list[dict[str, Any]] = [
    SEARCH_PRODUCTS_SCHEMA,
    LIST_FACETS_SCHEMA,
    SEMANTIC_SEARCH_SCHEMA,
    GET_PRODUCT_DETAILS_SCHEMA,
]
TOOL_NAMES: tuple[str, ...] = (
    "search_products",
    "list_facets",
    "semantic_search",
    "get_product_details",
)


class SearchProductsArgs(SearchFilters):
    """Validated arguments for `search_products`."""


class ListFacetsArgs(BaseModel):
    field: str = Field(...)

    @field_validator("field")
    @classmethod
    def _check_field(cls, v: str) -> str:
        if v not in FACET_FIELDS:
            raise ValueError(f"unknown facet '{v}'. Allowed: {', '.join(FACET_FIELDS)}")
        return v


class SemanticSearchArgs(BaseModel):
    query: str = Field(min_length=1, max_length=500)
    top_k: int = Field(default=10, ge=1, le=20)
    filters: SemanticSearchFilters = Field(default_factory=SemanticSearchFilters)


class GetProductDetailsArgs(BaseModel):
    id: UUID


class ToolResult(BaseModel):
    name: str
    payload: dict[str, Any]
    product_ids: list[str] = Field(default_factory=list)

    def as_json(self) -> str:
        return json.dumps(self.payload, ensure_ascii=False, default=str)


@dataclass
class ToolContext:
    """Bag of dependencies a tool dispatch may need."""

    product: ProductClient
    embedder: MultimodalEmbedder | None = None
    query_counter: QueryEmbedCounter | None = None
    # Forwarded to product-service so admin-only endpoints honour the
    # caller. For MVP only admin sessions reach the stylist chat, so
    # passing through the request's identity is enough.
    as_user: tuple[UUID, str] | None = None


def _parse_args(raw: str) -> dict[str, Any]:
    if not raw:
        return {}
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid JSON arguments: {exc.msg}") from exc
    if not isinstance(data, dict):
        raise ValueError("arguments must be a JSON object")
    return data


def _validation_error(name: str, exc: Exception) -> ToolResult:
    return ToolResult(name=name, payload={"error": "validation", "message": str(exc)})


def _upstream_error(name: str, exc: UpstreamUnavailable) -> ToolResult:
    return ToolResult(
        name=name,
        payload={"error": "upstream_unavailable", "message": exc.message, "retriable": True},
    )


async def _dispatch_search_products(args_raw: str, ctx: ToolContext) -> ToolResult:
    try:
        args = SearchProductsArgs.model_validate(_parse_args(args_raw))
    except (ValidationError, ValueError) as exc:
        return _validation_error("search_products", exc)
    try:
        items = await ctx.product.get_products(args, as_user=ctx.as_user)
    except UpstreamUnavailable as exc:
        return _upstream_error("search_products", exc)

    capped = items[:20]
    return ToolResult(
        name="search_products",
        payload={
            "items": [
                {
                    "id": str(p.id),
                    "name": p.name,
                    "brand": p.brand,
                    "category": p.category,
                    "price": p.price,
                    "currency": p.currency,
                    "tags": p.tags,
                }
                for p in capped
            ],
            "total": len(items),
            "truncated": len(items) > len(capped),
        },
        product_ids=[str(p.id) for p in capped],
    )


def _facet_values(field: str, options) -> list[str]:
    return list(getattr(options, field, []) or [])


async def _dispatch_list_facets(args_raw: str, ctx: ToolContext) -> ToolResult:
    try:
        args = ListFacetsArgs.model_validate(_parse_args(args_raw))
    except (ValidationError, ValueError) as exc:
        return _validation_error("list_facets", exc)
    try:
        options = await ctx.product.get_filter_options(as_user=ctx.as_user)
    except UpstreamUnavailable as exc:
        return _upstream_error("list_facets", exc)
    return ToolResult(
        name="list_facets",
        payload={"field": args.field, "values": _facet_values(args.field, options)},
    )


async def _dispatch_semantic_search(args_raw: str, ctx: ToolContext) -> ToolResult:
    if ctx.embedder is None:
        return ToolResult(
            name="semantic_search",
            payload={
                "error": "embed_disabled",
                "message": (
                    "Семантический поиск временно недоступен — "
                    "переключись на search_products."
                ),
                "retriable": False,
            },
        )
    try:
        args = SemanticSearchArgs.model_validate(_parse_args(args_raw))
    except (ValidationError, ValueError) as exc:
        return _validation_error("semantic_search", exc)

    if ctx.query_counter is not None and not await ctx.query_counter.try_consume(1):
        log.info("tool.semantic_search.cap_hit")
        return ToolResult(
            name="semantic_search",
            payload={
                "error": "embed_cap",
                "message": (
                    "Дневной лимит семантических запросов исчерпан — "
                    "используй search_products вместо semantic_search."
                ),
                "retriable": False,
            },
        )

    try:
        vector = await ctx.embedder.embed_text(args.query)
    except UpstreamUnavailable as exc:
        return _upstream_error("semantic_search", exc)
    except Exception as exc:
        log.warning("tool.semantic_search.embed_failed", error=str(exc))
        return ToolResult(
            name="semantic_search",
            payload={"error": "upstream_unavailable", "message": str(exc), "retriable": True},
        )

    try:
        resp = await ctx.product.semantic_search(
            vector=vector,
            top_k=args.top_k,
            filters=args.filters,
        )
    except UpstreamUnavailable as exc:
        return _upstream_error("semantic_search", exc)

    items_payload: list[dict[str, Any]] = []
    product_ids: list[str] = []
    for item in resp.items:
        items_payload.append(
            {
                "id": str(item.id),
                "name": item.name,
                "brand": item.brand,
                "category": item.category,
                "price": item.price,
                "score": round(item.score, 4),
                "score_breakdown": {
                    "text": round(item.score_breakdown.text, 4),
                    "image": round(item.score_breakdown.image, 4),
                    "best_image_idx": item.score_breakdown.best_image_idx,
                },
                "highlight_fields": item.highlight_fields,
            }
        )
        product_ids.append(str(item.id))
    return ToolResult(
        name="semantic_search",
        payload={"items": items_payload, "total": len(items_payload)},
        product_ids=product_ids,
    )


async def _dispatch_get_product_details(args_raw: str, ctx: ToolContext) -> ToolResult:
    try:
        args = GetProductDetailsArgs.model_validate(_parse_args(args_raw))
    except (ValidationError, ValueError) as exc:
        return _validation_error("get_product_details", exc)
    try:
        product = await ctx.product.get_product(args.id, as_user=ctx.as_user)
    except UpstreamUnavailable as exc:
        return _upstream_error("get_product_details", exc)

    return ToolResult(
        name="get_product_details",
        payload={
            "id": str(product.id),
            "name": product.name,
            "brand": product.brand,
            "category": product.category,
            "price": product.price,
            "currency": product.currency,
            "description": product.description,
            "tags": product.tags,
            "size": product.size,
            "condition": product.condition,
            "image_count": product.image_count,
        },
        product_ids=[str(product.id)],
    )


DISPATCHERS = {
    "search_products": _dispatch_search_products,
    "list_facets": _dispatch_list_facets,
    "semantic_search": _dispatch_semantic_search,
    "get_product_details": _dispatch_get_product_details,
}


async def dispatch_tool(
    name: str,
    args_raw: str,
    *,
    ctx: ToolContext | None = None,
    product: ProductClient | None = None,
) -> ToolResult:
    """Run a tool by name.

    `ctx` carries all the dependencies a tool may need (product client,
    embedder, query counter). Phase 3 callers can still pass `product=`
    directly — we build a context on the fly for them.
    """
    if ctx is None:
        if product is None:
            raise ValueError("dispatch_tool needs either ctx or product")
        ctx = ToolContext(product=product)
    fn = DISPATCHERS.get(name)
    if fn is None:
        return ToolResult(
            name=name,
            payload={"error": "unknown_tool", "message": f"unknown tool '{name}'"},
        )
    log.info("tool.dispatch", tool=name)
    return await fn(args_raw, ctx)
