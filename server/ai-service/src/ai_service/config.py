from functools import lru_cache
from typing import Literal

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    service_port: int = Field(default=8084, alias="SERVICE_PORT")
    log_level: Literal["debug", "info", "warning", "error"] = Field(
        default="info", alias="LOG_LEVEL"
    )

    openai_api_key: str = Field(default="", alias="OPENAI_API_KEY")
    openai_chat_model: str = Field(default="gpt-4o-mini", alias="OPENAI_CHAT_MODEL")
    openai_request_timeout_seconds: int = Field(
        default=30, alias="OPENAI_REQUEST_TIMEOUT_SECONDS"
    )

    multimodal_embed_provider: Literal["cohere"] = Field(
        default="cohere", alias="MULTIMODAL_EMBED_PROVIDER"
    )
    multimodal_embed_model: str = Field(
        default="embed-multilingual-v3.0", alias="MULTIMODAL_EMBED_MODEL"
    )
    multimodal_embed_dim: int = Field(default=1024, alias="MULTIMODAL_EMBED_DIM")
    multimodal_embed_api_key: str = Field(default="", alias="MULTIMODAL_EMBED_API_KEY")
    multimodal_embed_base_url: str = Field(default="", alias="MULTIMODAL_EMBED_BASE_URL")
    multimodal_embed_timeout_seconds: int = Field(
        default=15, alias="MULTIMODAL_EMBED_TIMEOUT_SECONDS"
    )

    product_service_url: str = Field(
        default="http://sc-product-service:8081", alias="PRODUCT_SERVICE_URL"
    )
    product_service_timeout_seconds: int = Field(
        default=5, alias="PRODUCT_SERVICE_TIMEOUT_SECONDS"
    )
    internal_api_token: str = Field(default="", alias="INTERNAL_API_TOKEN")

    max_history_messages: int = Field(default=30, alias="MAX_HISTORY_MESSAGES")
    history_token_budget: int = Field(default=6000, alias="HISTORY_TOKEN_BUDGET")
    max_tool_iterations: int = Field(default=4, alias="MAX_TOOL_ITERATIONS")
    max_tokens_per_reply: int = Field(default=800, alias="MAX_TOKENS_PER_REPLY")
    conversation_ttl_minutes: int = Field(default=30, alias="CONVERSATION_TTL_MINUTES")

    rate_limit_messages_per_min: int = Field(default=12, alias="RATE_LIMIT_MESSAGES_PER_MIN")
    rate_limit_tokens_per_day: int = Field(default=50000, alias="RATE_LIMIT_TOKENS_PER_DAY")
    query_embed_daily_cap: int = Field(default=10000, alias="QUERY_EMBED_DAILY_CAP")

    jwt_secret: str = Field(default="", alias="JWT_SECRET")

    langsmith_tracing: bool = Field(default=True, alias="LANGSMITH_TRACING")
    langsmith_api_key: str = Field(default="", alias="LANGSMITH_API_KEY")
    langsmith_project: str = Field(default="sc-ai-service-dev", alias="LANGSMITH_PROJECT")
    langsmith_endpoint: str = Field(
        default="https://api.smith.langchain.com", alias="LANGSMITH_ENDPOINT"
    )

    @property
    def langsmith_enabled(self) -> bool:
        """True only if tracing is on AND an API key is configured."""
        return self.langsmith_tracing and bool(self.langsmith_api_key)


@lru_cache(maxsize=1)
def get_settings() -> Settings:
    return Settings()
