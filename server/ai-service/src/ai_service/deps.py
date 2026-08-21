from fastapi import Request

from .config import Settings
from .domain.conversation import ConversationStore
from .embeddings.counter import QueryEmbedCounter
from .embeddings.multimodal_client import MultimodalEmbedder
from .llm.client import LLMClient
from .product.client import ProductClient


def get_settings_dep(request: Request) -> Settings:
    return request.app.state.settings  # type: ignore[no-any-return]


def get_store(request: Request) -> ConversationStore:
    return request.app.state.conversation_store  # type: ignore[no-any-return]


def get_llm_client(request: Request) -> LLMClient:
    return request.app.state.llm_client  # type: ignore[no-any-return]


def get_product_client(request: Request) -> ProductClient:
    return request.app.state.product_client  # type: ignore[no-any-return]


def get_embedder(request: Request) -> MultimodalEmbedder | None:
    return getattr(request.app.state, "embedder", None)


def get_query_counter(request: Request) -> QueryEmbedCounter:
    return request.app.state.query_counter  # type: ignore[no-any-return]
