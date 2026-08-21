from ai_service.config import Settings, get_settings


def test_defaults(monkeypatch) -> None:
    for key in (
        "OPENAI_API_KEY",
        "MULTIMODAL_EMBED_API_KEY",
        "INTERNAL_API_TOKEN",
        "JWT_SECRET",
    ):
        monkeypatch.delenv(key, raising=False)
    get_settings.cache_clear()
    s = Settings(_env_file=None)
    assert s.service_port == 8084
    assert s.openai_chat_model == "gpt-4o-mini"
    assert s.multimodal_embed_provider == "cohere"
    assert s.multimodal_embed_dim == 1024
    assert s.history_token_budget == 6000


def test_env_overrides(monkeypatch) -> None:
    monkeypatch.setenv("SERVICE_PORT", "9090")
    monkeypatch.setenv("OPENAI_CHAT_MODEL", "gpt-4o")
    monkeypatch.setenv("MAX_TOOL_ITERATIONS", "7")
    get_settings.cache_clear()
    s = Settings(_env_file=None)
    assert s.service_port == 9090
    assert s.openai_chat_model == "gpt-4o"
    assert s.max_tool_iterations == 7


def test_langsmith_enabled_requires_key(monkeypatch) -> None:
    monkeypatch.setenv("LANGSMITH_TRACING", "true")
    monkeypatch.delenv("LANGSMITH_API_KEY", raising=False)
    get_settings.cache_clear()
    assert Settings(_env_file=None).langsmith_enabled is False


def test_langsmith_enabled_when_key_set(monkeypatch) -> None:
    monkeypatch.setenv("LANGSMITH_TRACING", "true")
    monkeypatch.setenv("LANGSMITH_API_KEY", "ls-secret")
    get_settings.cache_clear()
    assert Settings(_env_file=None).langsmith_enabled is True


def test_langsmith_disabled_explicitly(monkeypatch) -> None:
    monkeypatch.setenv("LANGSMITH_TRACING", "false")
    monkeypatch.setenv("LANGSMITH_API_KEY", "ls-secret")
    get_settings.cache_clear()
    assert Settings(_env_file=None).langsmith_enabled is False
