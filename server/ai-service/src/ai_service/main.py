import os
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager

import httpx
import structlog
from fastapi import APIRouter, Depends, FastAPI

from .auth import User, current_user
from .circuit_breaker import CircuitBreaker
from .config import Settings, get_settings
from .domain.conversation import ConversationStore
from .embeddings.counter import QueryEmbedCounter
from .embeddings.multimodal_client import CohereEmbedder, EmbedError
from .handlers import chat as chat_handler
from .handlers import conversations as conversations_handler
from .llm.client import LLMClient
from .observability import install_observability
from .product.client import ProductClient
from .ratelimit import RateLimiter

log = structlog.get_logger(__name__)


def _propagate_langsmith_env(settings: Settings) -> None:
    """Ensure the langsmith SDK sees vars even when loaded from a .env file."""
    os.environ["LANGSMITH_TRACING"] = "true" if settings.langsmith_tracing else "false"
    if settings.langsmith_api_key:
        os.environ["LANGSMITH_API_KEY"] = settings.langsmith_api_key
    if settings.langsmith_project:
        os.environ["LANGSMITH_PROJECT"] = settings.langsmith_project
    if settings.langsmith_endpoint:
        os.environ["LANGSMITH_ENDPOINT"] = settings.langsmith_endpoint


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncIterator[None]:
    settings: Settings = app.state.settings
    _propagate_langsmith_env(settings)

    store = ConversationStore(
        ttl_minutes=settings.conversation_ttl_minutes,
        max_messages=settings.max_history_messages,
    )
    await store.start()
    app.state.conversation_store = store

    openai_breaker = CircuitBreaker("openai")
    cohere_breaker = CircuitBreaker("cohere")
    product_breaker = CircuitBreaker("product_service")
    app.state.openai_breaker = openai_breaker
    app.state.cohere_breaker = cohere_breaker
    app.state.product_breaker = product_breaker

    app.state.rate_limiter = RateLimiter(
        messages_per_min=settings.rate_limit_messages_per_min,
        tokens_per_day=settings.rate_limit_tokens_per_day,
    )

    llm_client = LLMClient(
        api_key=settings.openai_api_key,
        model=settings.openai_chat_model,
        timeout_seconds=settings.openai_request_timeout_seconds,
        langsmith_enabled=settings.langsmith_enabled,
        breaker=openai_breaker,
    )
    app.state.llm_client = llm_client

    http_client = httpx.AsyncClient(
        timeout=settings.product_service_timeout_seconds,
        limits=httpx.Limits(max_connections=64, max_keepalive_connections=16),
    )
    app.state.http_client = http_client
    app.state.product_client = ProductClient(
        http=http_client,
        base_url=settings.product_service_url,
        internal_token=settings.internal_api_token,
        breaker=product_breaker,
    )

    app.state.query_counter = QueryEmbedCounter(daily_cap=settings.query_embed_daily_cap)
    embedder = None
    if settings.multimodal_embed_api_key:
        try:
            embedder = CohereEmbedder(
                api_key=settings.multimodal_embed_api_key,
                model=settings.multimodal_embed_model,
                dim=settings.multimodal_embed_dim,
                base_url=settings.multimodal_embed_base_url or None,
                timeout_seconds=settings.multimodal_embed_timeout_seconds,
                breaker=cohere_breaker,
            )
        except EmbedError as exc:
            log.warning("ai_service.embedder_disabled", error=str(exc))
    else:
        log.info("ai_service.embedder_disabled", reason="no_api_key")
    app.state.embedder = embedder

    log.info(
        "ai_service.startup",
        port=settings.service_port,
        model=settings.openai_chat_model,
        langsmith=settings.langsmith_enabled,
        langsmith_project=settings.langsmith_project if settings.langsmith_enabled else None,
        product_service=settings.product_service_url,
        embedder_enabled=embedder is not None,
        embed_model=settings.multimodal_embed_model if embedder is not None else None,
    )
    try:
        yield
    finally:
        log.info("ai_service.shutdown")
        await http_client.aclose()
        await store.stop()


def create_app() -> FastAPI:
    settings = get_settings()
    app = FastAPI(
        title="ai-service",
        version="0.1.0",
        lifespan=lifespan,
        docs_url="/docs",
        redoc_url=None,
    )
    app.state.settings = settings
    install_observability(app, settings.log_level)

    @app.get("/healthz", include_in_schema=False)
    async def healthz() -> dict[str, str]:
        return {"status": "ok"}

    stylist = APIRouter(prefix="/api/stylist", tags=["stylist"])
    user_dep = Depends(current_user)

    @stylist.get("/whoami")
    async def whoami(user: User = user_dep) -> dict[str, str]:
        return {"user_id": str(user.id), "role": user.role}

    app.include_router(stylist)
    app.include_router(conversations_handler.router)
    app.include_router(chat_handler.router)
    return app


app = create_app()


__all__ = ["app", "create_app", "Settings"]
