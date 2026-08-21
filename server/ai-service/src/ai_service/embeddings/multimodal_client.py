"""Multimodal embedder for the chat side.

`ai-service` only ever needs query embeddings (`search_query`), so this
file is intentionally narrower than the Rust client in `product-service`.
Both must share the same model and dim — otherwise the query vector
ends up in a different space and hybrid search returns nonsense.
"""

from __future__ import annotations

import asyncio
import random
import time
from typing import Protocol

import structlog

from ..circuit_breaker import CircuitBreaker, CircuitOpen
from ..domain.errors import UpstreamUnavailable
from ..metrics import (
    circuit_breaker_rejections_total,
    embed_request_duration_seconds,
    embed_requests_total,
    record_breaker_state,
)

log = structlog.get_logger(__name__)


class EmbedError(Exception):
    pass


class MultimodalEmbedder(Protocol):
    @property
    def dim(self) -> int: ...

    async def embed_text(self, text: str) -> list[float]: ...


class CohereEmbedder:
    """Cohere v2 embed client for `search_query` text vectors."""

    def __init__(
        self,
        *,
        api_key: str,
        model: str,
        dim: int,
        base_url: str | None = None,
        timeout_seconds: float = 15.0,
        max_retries: int = 2,
        breaker: CircuitBreaker | None = None,
    ) -> None:
        if not api_key:
            raise EmbedError("MULTIMODAL_EMBED_API_KEY is empty")
        # Imported lazily so tests can stub the client without installing cohere.
        from cohere import AsyncClientV2

        kwargs: dict[str, object] = {"api_key": api_key, "timeout": timeout_seconds}
        if base_url:
            kwargs["base_url"] = base_url
        self._client = AsyncClientV2(**kwargs)  # type: ignore[arg-type]
        self._model = model
        self._dim = dim
        self._max_retries = max_retries
        self._breaker = breaker

    @property
    def dim(self) -> int:
        return self._dim

    async def embed_text(self, text: str) -> list[float]:
        if not text:
            raise EmbedError("embed_text called with empty string")

        if self._breaker is not None:
            try:
                await self._breaker.check()
            except CircuitOpen:
                circuit_breaker_rejections_total.labels(name="cohere").inc()
                record_breaker_state("cohere", self._breaker.state.value)
                raise UpstreamUnavailable("cohere breaker open") from None

        last_exc: Exception | None = None
        started = time.monotonic()
        for attempt in range(1, self._max_retries + 2):
            try:
                resp = await self._client.embed(
                    model=self._model,
                    input_type="search_query",
                    texts=[text],
                    embedding_types=["float"],
                )
            except Exception as exc:  # cohere wraps errors in its own classes
                last_exc = exc
                msg = str(exc).lower()
                log.warning(
                    "cohere.embed_attempt_failed",
                    attempt=attempt,
                    error=str(exc),
                )
                # 429 / 5xx — retry; anything else fails fast.
                if not any(t in msg for t in ("429", "5", "timeout", "temporarily")):
                    await self._mark_failure()
                    embed_requests_total.labels(provider="cohere", result="error").inc()
                    raise UpstreamUnavailable(f"cohere embed failed: {exc}") from exc
                if attempt > self._max_retries:
                    break
                await asyncio.sleep(min(2 ** (attempt - 1), 4) + random.uniform(0, 0.2))
                continue

            vectors = _extract_float_vectors(resp)
            if not vectors:
                await self._mark_failure()
                embed_requests_total.labels(provider="cohere", result="error").inc()
                raise UpstreamUnavailable("cohere returned no vectors")
            vec = vectors[0]
            if len(vec) != self._dim:
                await self._mark_failure()
                embed_requests_total.labels(provider="cohere", result="error").inc()
                raise UpstreamUnavailable(
                    f"cohere vector dim {len(vec)} != configured {self._dim}"
                )
            embed_request_duration_seconds.labels(provider="cohere").observe(
                time.monotonic() - started
            )
            embed_requests_total.labels(provider="cohere", result="ok").inc()
            await self._mark_success()
            return vec

        await self._mark_failure()
        embed_requests_total.labels(provider="cohere", result="error").inc()
        raise UpstreamUnavailable("cohere embed unavailable after retries") from last_exc

    async def _mark_success(self) -> None:
        if self._breaker is None:
            return
        await self._breaker.record_success()
        record_breaker_state("cohere", self._breaker.state.value)

    async def _mark_failure(self) -> None:
        if self._breaker is None:
            return
        await self._breaker.record_failure()
        record_breaker_state("cohere", self._breaker.state.value)


def _extract_float_vectors(resp: object) -> list[list[float]]:
    """SDK shape differs across versions — defensively pull `.float`/`float_`."""
    embeddings = getattr(resp, "embeddings", None)
    if embeddings is None and isinstance(resp, dict):
        embeddings = resp.get("embeddings")
    if embeddings is None:
        return []
    for attr in ("float", "float_"):
        vectors = getattr(embeddings, attr, None)
        if vectors is None and isinstance(embeddings, dict):
            vectors = embeddings.get(attr)
        if vectors:
            return [list(v) for v in vectors]
    return []
