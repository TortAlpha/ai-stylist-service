import asyncio
import random
from typing import Any
from uuid import UUID

import httpx
import structlog

from ..circuit_breaker import CircuitBreaker, CircuitOpen
from ..domain.errors import UpstreamUnavailable
from ..metrics import (
    circuit_breaker_rejections_total,
    product_service_requests_total,
    record_breaker_state,
)
from .schemas import (
    FilterOptions,
    ProductDetails,
    ProductSummary,
    SearchFilters,
    SemanticSearchFilters,
    SemanticSearchResponse,
)

log = structlog.get_logger(__name__)

# Static enums product-service doesn't expose as endpoints — keep in sync
# with `migrations/init/02_product_schema.sql`.
_STATIC_CONDITIONS = ["new", "like_new", "good", "fair", "poor"]
_STATIC_SIZES: list[str] = []  # sizes are per-category in this schema


class ProductClient:
    """Async client to product-service.

    Retries 5xx and timeouts with exponential backoff + jitter.
    Maps persistent failures to UpstreamUnavailable so the LLM loop can
    surface them as tool errors instead of crashing the SSE stream.

    product-service currently exposes only admin-scoped endpoints for
    listing/details (`/api/admin/products/*`). Chat handler forwards the
    requesting user's `X-User-Id`/`X-User-Role` headers via `as_user=`
    so admin sessions can read; a customer call here would 401/403 —
    fine for MVP since only admins use the stylist chat right now.
    """

    def __init__(
        self,
        *,
        http: httpx.AsyncClient,
        base_url: str,
        internal_token: str = "",
        max_retries: int = 2,
        breaker: CircuitBreaker | None = None,
    ) -> None:
        self._http = http
        self._base_url = base_url.rstrip("/")
        self._internal_token = internal_token
        self._max_retries = max_retries
        self._breaker = breaker

    def _url(self, path: str) -> str:
        return f"{self._base_url}{path}"

    def _headers(
        self,
        *,
        internal: bool = False,
        as_user: tuple[UUID, str] | None = None,
    ) -> dict[str, str]:
        headers: dict[str, str] = {"Accept": "application/json"}
        if internal and self._internal_token:
            headers["X-Internal-Token"] = self._internal_token
        if as_user is not None:
            uid, role = as_user
            headers["X-User-Id"] = str(uid)
            headers["X-User-Role"] = role
        return headers

    async def _request_json(
        self,
        method: str,
        path: str,
        *,
        params: dict[str, str] | None = None,
        json_body: dict | None = None,
        internal: bool = False,
        as_user: tuple[UUID, str] | None = None,
    ) -> Any:
        if self._breaker is not None:
            try:
                await self._breaker.check()
            except CircuitOpen:
                circuit_breaker_rejections_total.labels(name="product_service").inc()
                record_breaker_state("product_service", self._breaker.state.value)
                raise UpstreamUnavailable("product-service breaker open") from None

        last_exc: Exception | None = None
        for attempt in range(1, self._max_retries + 2):
            try:
                resp = await self._http.request(
                    method,
                    self._url(path),
                    params=params,
                    json=json_body,
                    headers=self._headers(internal=internal, as_user=as_user),
                )
            except (httpx.TimeoutException, httpx.TransportError) as exc:
                last_exc = exc
                product_service_requests_total.labels(endpoint=path, status="transport").inc()
                log.warning(
                    "product_client.transport_error",
                    method=method,
                    path=path,
                    attempt=attempt,
                    error=str(exc),
                )
            else:
                if 200 <= resp.status_code < 300:
                    product_service_requests_total.labels(endpoint=path, status="ok").inc()
                    await self._mark_success()
                    return resp.json()
                if 500 <= resp.status_code < 600:
                    last_exc = httpx.HTTPStatusError(
                        f"{resp.status_code}", request=resp.request, response=resp
                    )
                    product_service_requests_total.labels(endpoint=path, status="5xx").inc()
                    log.warning(
                        "product_client.5xx",
                        path=path,
                        status=resp.status_code,
                        attempt=attempt,
                    )
                else:
                    product_service_requests_total.labels(endpoint=path, status="4xx").inc()
                    log.warning(
                        "product_client.4xx",
                        path=path,
                        status=resp.status_code,
                    )
                    await self._mark_failure()
                    raise UpstreamUnavailable(
                        f"product-service {resp.status_code}"
                    )

            if attempt > self._max_retries:
                break
            await asyncio.sleep(min(2 ** (attempt - 1), 4) + random.uniform(0, 0.2))

        await self._mark_failure()
        raise UpstreamUnavailable("product-service unavailable") from last_exc

    async def _mark_success(self) -> None:
        if self._breaker is None:
            return
        await self._breaker.record_success()
        record_breaker_state("product_service", self._breaker.state.value)

    async def _mark_failure(self) -> None:
        if self._breaker is None:
            return
        await self._breaker.record_failure()
        record_breaker_state("product_service", self._breaker.state.value)

    @staticmethod
    def _unwrap_data(payload: Any) -> Any:
        """product-service wraps payloads in `ApiResponse{success,data,...}`."""
        if isinstance(payload, dict) and "data" in payload and "success" in payload:
            return payload["data"]
        return payload

    @staticmethod
    def _coerce_price(value: Any) -> int | None:
        if value is None:
            return None
        try:
            return int(float(value))
        except (TypeError, ValueError):
            return None

    async def get_products(
        self,
        filters: SearchFilters,
        *,
        as_user: tuple[UUID, str] | None = None,
    ) -> list[ProductSummary]:
        payload = await self._request_json(
            "GET",
            "/api/admin/products",
            params=filters.to_query(),
            as_user=as_user,
        )
        data = self._unwrap_data(payload)
        items = data.get("items", []) if isinstance(data, dict) else (data or [])
        result: list[ProductSummary] = []
        for raw in items:
            try:
                result.append(
                    ProductSummary(
                        id=raw["id"],
                        name=raw.get("name", ""),
                        brand=raw.get("brand_name") or raw.get("brand"),
                        category=raw.get("category"),
                        price=self._coerce_price(raw.get("purchase_price")),
                        currency=raw.get("currency"),
                        tags=raw.get("tags", []) if isinstance(raw.get("tags"), list) else [],
                        image_count=raw.get("image_count", 0) or 0,
                    )
                )
            except (KeyError, ValueError) as exc:
                log.warning("product_client.parse_summary_failed", error=str(exc))
        return result

    async def get_product(
        self,
        product_id: UUID,
        *,
        as_user: tuple[UUID, str] | None = None,
    ) -> ProductDetails:
        payload = await self._request_json(
            "GET",
            f"/api/admin/products/{product_id}",
            as_user=as_user,
        )
        data = self._unwrap_data(payload)
        if not isinstance(data, dict):
            raise UpstreamUnavailable("product-service: unexpected product payload")

        brand = data.get("brand")
        brand_name = brand.get("name") if isinstance(brand, dict) else brand
        category = data.get("category")
        category_name = category.get("name") if isinstance(category, dict) else category

        details = data.get("details") or {}
        tags_obj = data.get("tags") or {}
        tag_pool: list[str] = []
        for key in ("styles", "vibes", "seasons"):
            for v in tags_obj.get(key, []) or []:
                if isinstance(v, str):
                    tag_pool.append(v)

        size_obj = data.get("size") or {}
        size_str = size_obj.get("size_value") or size_obj.get("label") or None

        return ProductDetails(
            id=data["id"],
            name=data.get("name", ""),
            brand=brand_name,
            category=category_name,
            price=self._coerce_price(data.get("purchase_price")),
            currency=data.get("currency"),
            description=details.get("special_notes") or data.get("ai_notes"),
            tags=tag_pool,
            size=size_str,
            condition=details.get("condition"),
            image_count=len(data.get("image_urls") or []),
        )

    async def get_filter_options(
        self,
        *,
        as_user: tuple[UUID, str] | None = None,
    ) -> FilterOptions:
        """Build a unified facet view from public lookup endpoints.

        `/api/admin/products/filter-options` returns ids; for tools we
        want names. Brands/categories/tags have public list endpoints we
        can read without admin role; sizes/conditions fall back to the
        static enum because they're not exposed as a flat list.
        """
        brands_payload = await self._request_json("GET", "/api/brands")
        categories_payload = await self._request_json("GET", "/api/categories")
        styles_payload = await self._request_json("GET", "/api/tags/styles")
        vibes_payload = await self._request_json("GET", "/api/tags/vibes")
        seasons_payload = await self._request_json("GET", "/api/tags/seasons")

        return FilterOptions(
            brands=_names_from(self._unwrap_data(brands_payload)),
            categories=_names_from(self._unwrap_data(categories_payload)),
            tags=(
                _names_from(self._unwrap_data(styles_payload))
                + _names_from(self._unwrap_data(vibes_payload))
                + _names_from(self._unwrap_data(seasons_payload))
            ),
            sizes=list(_STATIC_SIZES),
            conditions=list(_STATIC_CONDITIONS),
        )

    async def semantic_search(
        self,
        *,
        vector: list[float],
        top_k: int,
        filters: SemanticSearchFilters,
    ) -> SemanticSearchResponse:
        body = {
            "vector": vector,
            "top_k": top_k,
            "filters": filters.model_dump(),
        }
        payload = await self._request_json(
            "POST",
            "/api/internal/products/semantic-search",
            json_body=body,
            internal=True,
        )
        # internal endpoint returns the raw response (no ApiResponse wrap).
        return SemanticSearchResponse.model_validate(payload)


def _names_from(payload: Any) -> list[str]:
    if isinstance(payload, dict):
        payload = payload.get("items") or []
    if not isinstance(payload, list):
        return []
    out: list[str] = []
    for raw in payload:
        if isinstance(raw, dict):
            name = raw.get("name")
            if isinstance(name, str) and name:
                out.append(name)
        elif isinstance(raw, str):
            out.append(raw)
    return out
