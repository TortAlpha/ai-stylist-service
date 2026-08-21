"""Prometheus metrics matching `DESIGN.md → Наблюдаемость`.

Importing this module is enough to register the metrics in the default
registry — they show up at `/metrics` once `observability.install_observability`
mounts the endpoint.
"""

from __future__ import annotations

from prometheus_client import Counter, Gauge, Histogram

# Chat-level outcome / latency.
chat_messages_total = Counter(
    "stylist_chat_messages_total",
    "Chat messages handled, labelled by terminal result.",
    labelnames=("result",),  # ok | rate_limited | upstream_unavailable | bad_request | internal
)
chat_duration_seconds = Histogram(
    "stylist_chat_duration_seconds",
    "Wall time from POST /chat receipt to event: done emitted.",
)
active_conversations = Gauge(
    "stylist_active_conversations",
    "In-memory conversations currently held by the store.",
)

# Tool-use loop.
tool_calls_total = Counter(
    "stylist_tool_calls_total",
    "LLM tool dispatches, labelled by tool name and outcome.",
    labelnames=("tool", "result"),  # ok | validation | upstream | unknown
)

# OpenAI accounting.
openai_tokens_total = Counter(
    "stylist_openai_tokens_total",
    "OpenAI tokens consumed.",
    labelnames=("model", "kind"),  # kind: prompt | completion | total
)
openai_request_duration_seconds = Histogram(
    "stylist_openai_request_duration_seconds",
    "OpenAI chat.completions.create wall time.",
    labelnames=("model", "kind"),  # kind: stream | nostream
)

# Cohere / multimodal embeddings.
embed_requests_total = Counter(
    "stylist_embed_requests_total",
    "Multimodal embedder calls.",
    labelnames=("provider", "result"),  # result: ok | error
)
embed_request_duration_seconds = Histogram(
    "stylist_embed_request_duration_seconds",
    "Multimodal embedder wall time.",
    labelnames=("provider",),
)

# product-service.
product_service_requests_total = Counter(
    "stylist_product_service_requests_total",
    "Requests to product-service.",
    labelnames=("endpoint", "status"),  # status: ok | 4xx | 5xx | transport
)

# Rate limiting + circuit breaker.
rate_limit_hits_total = Counter(
    "stylist_rate_limit_hits_total",
    "Times a user hit a rate limit.",
    labelnames=("reason",),  # messages_per_min | tokens_per_day
)
circuit_breaker_state = Gauge(
    "stylist_circuit_breaker_state",
    "Circuit breaker state: 0=closed, 1=half_open, 2=open.",
    labelnames=("name",),
)
circuit_breaker_rejections_total = Counter(
    "stylist_circuit_breaker_rejections_total",
    "Calls short-circuited by an open breaker.",
    labelnames=("name",),
)


def record_breaker_state(name: str, state: str) -> None:
    value = {"closed": 0, "half_open": 1, "open": 2}.get(state, 0)
    circuit_breaker_state.labels(name=name).set(value)
