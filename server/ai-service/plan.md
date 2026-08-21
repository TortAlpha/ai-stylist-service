# ai-service — план реализации

План написан после анализа `DESIGN.md`, `ROADMAP.md`, `RESP_SPLIT.md` и фактического состояния каталога `server/ai-service/`. Цель — превратить пустой скелет в работающий stylist-сервис (Python + FastAPI), пошагово, мерж за мержем.

Документы-первоисточники остаются авторитетными:
- **Что строим и почему** — `DESIGN.md`.
- **В каком порядке и критерии «готово»** — `ROADMAP.md`.
- **Кто за что отвечает (FE/BE/QA)** — `RESP_SPLIT.md`.

Здесь — только последовательность работ BE-инженера по самому `ai-service` плюс минимально необходимое в `product-service`/`nginx`/корневом compose.

---

## 0. Текущее состояние (что уже есть)

- Каталог `server/ai-service/` создан, структура из `DESIGN.md → Структура каталога` развёрнута.
- Все `.py`-файлы в `src/ai_service/` — **пустые** (`__init__.py`, `main.py`, `config.py`, `auth.py`, `deps.py`, `observability.py`, `ratelimit.py`, и в подкаталогах `domain/`, `handlers/`, `llm/`, `product/`).
- `pyproject.toml` — без зависимостей, только пакет-обвязка.
- `Dockerfile` — заглушка (`FROM python:3.12-slim`, `WORKDIR /app`, нет ни pip install, ни CMD).
- `assets/docker-compose.ai.yml` и `docker-compose.ai.prod.yml` — стабы с TODO.
- `.env.example` — заполнен по `DESIGN.md → Конфигурация` (включая Cohere-вариант).
- `migrations/` — пусто (на MVP миграции не нужны, in-memory state).
- `tests/{unit,integration}/` — пусто.
- В корневом `docker-compose.yml` секции `sc-ai-service` ещё **нет**.
- В `nginx/nginx.conf` блока `/api/stylist/` ещё **нет** (есть только `auth`/`products`/`brands`/...).
- В `product-service` нет `pgvector`, `MultimodalEmbedder`, ни таблиц `product_text_embeddings`/`product_image_embeddings`, ни internal-эндпоинта `POST /internal/products/semantic-search`.

Зафиксированные решения, на которые опираемся:
- Multimodal embedder: **Cohere `embed-multilingual-v3.0`, dim 1024** (REST), один и тот же в `ai-service` и `product-service` — иначе query-вектор окажется в чужом пространстве.
- Chat-модель: `gpt-4o-mini` по умолчанию, эскалация на `gpt-4o` через env.
- JWT терминируется на nginx через `auth_request → /internal/auth/verify`, `ai-service` доверяет `X-User-Id` / `X-User-Role`.
- История диалогов и rate-limit — **in-memory** (post-MVP — Redis).

---

## 1. Карта фаз (то, как пойдём)

```
P0a (PS, text) ─┐
                ├─► P4 (AI hybrid semantic + drill-in)
P0b (PS, image)─┘
P1 (AI каркас) ─► P2 (chat без tools) ─► P3 (точечный поиск) ─► P4 ─► P5 (лимиты/устойчивость) ─► P6 (релиз)
```

- **P0a и P1 можно вести параллельно.** P0a/P0b ведутся в `product-service`, не блокируют каркас `ai-service`.
- **P2, P3 — строго после P1.** Каждая следующая опирается на код предыдущей.
- **P4 ждёт P0a + P0b + P3.**
- **P5 → P6** — последовательно.

Каждая фаза мержится отдельным PR. До мержа — критерии «Готово, когда» из `ROADMAP.md` и приёмка QA.

---

## 2. Фаза 1 — каркас ai-service (без LLM)

**Зачем:** поднять процесс, прокинуть через nginx с auth, иметь `/healthz` и `/metrics`. Без этой базы дальше делать нечего.

### 2.1 `pyproject.toml` — зависимости

Добавить runtime-зависимости:
```
fastapi, uvicorn[standard], httpx, pydantic, pydantic-settings,
sse-starlette, structlog, prometheus_client, openai>=1.30, tiktoken
```
Dev-группа:
```
pytest, pytest-asyncio, pytest-httpserver, ruff, mypy
```
SDK `cohere` подключим в Phase 4 вместе с `multimodal_client.py` — раньше он не нужен.

### 2.2 `src/ai_service/config.py`

`pydantic-settings`:
- Все ключи из `.env.example` отображаются в типизированные поля.
- Группы: `OpenAISettings`, `MultimodalEmbedSettings`, `ProductServiceSettings`, `HistorySettings`, `RateLimitSettings`, `AuthSettings`.
- Один глобальный `get_settings()` через `lru_cache`.
- Валидация на старте: если `OPENAI_API_KEY` пуст и `LOG_LEVEL != debug` — ругаемся (для dev можно через переменную обхода).

### 2.3 `src/ai_service/observability.py`

- `configure_logging(level)` — `structlog` JSON-логер, ProcessorFormatter под stdlib `logging`.
- `RequestIdMiddleware` — берёт `X-Request-Id` от nginx или генерит ULID, пробрасывает в `contextvars`.
- `metrics_router` — отдаёт `/metrics` через `prometheus_client.make_asgi_app()`.

### 2.4 `src/ai_service/auth.py`

- `current_user(request)` FastAPI-зависимость: читает `X-User-Id`, `X-User-Role`. Если хедера нет → `HTTPException(401)`.
- Опциональный JWT verify (`JWT_SECRET`) — defence-in-depth, выкл по умолчанию.
- `User` Pydantic-модель (`id: UUID`, `role: Literal["customer","admin"]`).

### 2.5 `src/ai_service/main.py`

- `app = FastAPI(lifespan=lifespan)`.
- В `lifespan`: создаём async `httpx.AsyncClient` к `product-service` и кладём в `app.state` (closing в shutdown).
- Подключаем `RequestIdMiddleware`, `/healthz` (200, без внешних зависимостей), `/metrics`.
- Заглушка `GET /api/stylist/whoami` → возвращает `{"user_id": user.id, "role": user.role}` (нужна для проверки auth-цепочки).

### 2.6 `Dockerfile`

Multi-stage:
- `builder`: `python:3.12-slim`, ставим `uv` или просто `pip wheel`, кешируем слой зависимостей отдельно от кода.
- `runtime`: `python:3.12-slim`, неприв. пользователь, копируем wheels и код, `CMD ["uvicorn","ai_service.main:app","--host","0.0.0.0","--port","8084"]`.
- `HEALTHCHECK` на `/healthz`.

### 2.7 `assets/docker-compose.ai.yml`

- build из `..`, env_file `.env`, порт 8084 наружу только в dev.
- `depends_on`: `sc-auth-service` (healthy), `sc-product-service` (healthy).
- Healthcheck: `curl -f http://localhost:8084/healthz || exit 1`.

### 2.8 Корневой `docker-compose.yml`

- Добавить блок `sc-ai-service` через `extends` (по образцу `sc-product-service`).
- В `nginx.depends_on` добавить `sc-ai-service`.

### 2.9 `nginx/nginx.conf`

Добавить блок (по DESIGN.md → Деплой):
```
upstream ai-service { server sc-ai-service:8084; }

location /api/stylist/ {
    auth_request     /internal/auth/verify;
    auth_request_set $user_id   $upstream_http_x_user_id;
    auth_request_set $user_role $upstream_http_x_user_role;

    proxy_pass http://ai-service;
    proxy_set_header Host              $host;
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header X-User-Id         $user_id;
    proxy_set_header X-User-Role       $user_role;

    proxy_buffering           off;
    proxy_cache               off;
    proxy_http_version        1.1;
    proxy_set_header Connection "";
    proxy_read_timeout        600s;
    proxy_send_timeout        600s;
}
```

### 2.10 Тесты

- `tests/unit/test_config.py` — настройки читаются из env, валидация ошибок.
- `tests/unit/test_auth.py` — `whoami` без `X-User-Id` → 401, с заголовками → 200.
- `tests/integration/test_healthz.py` — `/healthz` отвечает 200 без внешних сервисов.

### 2.11 Критерии готовности (Phase 1)

- `docker compose up` поднимается до healthy.
- `curl http://localhost/api/stylist/whoami` без JWT → 401; с валидным JWT → 200 с `user_id`.
- `/metrics` отдаёт прометеусный текст.
- `/healthz` не зависит от OpenAI / product-service.

---

## 3. Фаза 2 — chat без tools (SSE, без подбора товаров)

**Зачем:** вертикальный срез «пользователь → OpenAI → стрим обратно». История in-memory, бюджет работает, lifecycle сессии работает. Tool-use отложен.

### 3.1 `domain/conversation.py`

- `Message` (`role`, `content`, `tool_calls?`, `tool_call_id?`, `pruned: bool`, `created_at`).
- `Conversation` (`id: UUID`, `user_id`, `messages: list[Message]`, `last_product_ids: list[UUID]`, `created_at`, `last_active`).
- `ConversationStore`:
  - in-memory `dict[UUID, Conversation]`, защищён `asyncio.Lock` per conversation.
  - TTL: фоновая `asyncio.Task`, раз в минуту чистит истёкшие (`CONVERSATION_TTL_MINUTES`).
  - `MAX_HISTORY_MESSAGES` — hard cap (за ним — суммаризация в Phase 2.4).

### 3.2 `domain/errors.py`

- `RateLimited`, `UpstreamUnavailable`, `BadRequest`, `Internal` — соответствуют `code` из SSE-протокола.

### 3.3 `llm/client.py`

- Тонкая обёртка над `openai.AsyncOpenAI`.
- `chat_stream(messages, *, tools=None, tool_choice=None, max_tokens) -> AsyncIterator[ChatChunk]` — рейзит `RateLimited`/`UpstreamUnavailable` по кодам OpenAI.
- Retry: `tenacity` или ручной экспоненциальный backoff с jitter, 3 попытки на 429/5xx, не дольше `OPENAI_REQUEST_TIMEOUT_SECONDS`.

### 3.4 `llm/prompts.py`

- `system_prompt_v0(user_role)` — версия БЕЗ описания tools (Phase 2 их ещё нет). Текст из `DESIGN.md → Системный промпт`, обрезанный до общих правил («отвечай по-русски», «не выдумывай товары»).

### 3.5 `llm/budget.py`

- `count_tokens(model, messages)` через `tiktoken`.
- `trim_history(messages, *, budget=HISTORY_TOKEN_BUDGET)` — откидывает старые сообщения, пока сумма не помещается.
- `summarize_dropped(client, dropped)` — синхронный cheap-вызов `gpt-4o-mini`, `max_tokens=200`, возвращает одно «system: summary» сообщение. Вызывается, только если откинули ≥ 3 сообщения.

### 3.6 `handlers/conversations.py`

- `POST /api/stylist/conversations` → создаёт пустой `Conversation`, возвращает `{"conversation_id": "..."}`.
- `DELETE /api/stylist/conversations/{id}` → удаляет, 204; идемпотентно (для несуществующего тоже 204).

### 3.7 `handlers/chat.py`

- `POST /api/stylist/chat` → SSE через `sse_starlette.EventSourceResponse(ping=15)`.
- Алгоритм (без tools):
  1. Валидируем `conversation_id`, тело, ищем conversation; если нет — `event: error code=bad_request`, `event: done`.
  2. Кладём `Message(role="user", ...)` в conversation.
  3. Собираем prompt = `[system_v0, ...trim_history(...)]`.
  4. `async for chunk in llm.chat_stream(...)` → `event: token data: {"text": ...}`.
  5. Собираем ответ целиком, кладём `Message(role="assistant", ...)` в conversation.
  6. `event: done`.
- Ошибки оборачиваем в `event: error code=...` и **всегда** последним шлём `event: done`.

### 3.8 Тесты

- `tests/unit/test_budget.py` — `trim_history` режет по бюджету, `summarize_dropped` зовётся при ≥ 3 откинутых.
- `tests/unit/test_conversation_store.py` — TTL, MAX_HISTORY_MESSAGES, конкурентный доступ.
- `tests/integration/test_chat_stream.py` — OpenAI замокан (response в виде асинхронного итератора чанков), новый conv → `event: token` ≥ 1 → `event: done`.
- Ручной smoke: `curl -N` чат «привет» → видно поток токенов.

### 3.9 Критерии готовности (Phase 2)

- В браузере / `curl -N` виден поток токенов.
- При рестарте контейнера история ожидаемо пропадает (документируем в `README.md`).
- Срабатывает обрезка истории: после 30+ сообщений в логах виден `[summary]` префикс.

---

## 4. Фаза 3 — точечный поиск (`search_products` + `list_facets`)

**Зачем:** первый полезный сценарий — Пример 1 из `DESIGN.md` («Покажи кожаные сумки до 10к»).

### 4.1 `product/schemas.py`

Pydantic-модели под DTO `product-service`:
- `ProductSummary` (id, name, brand, category, price, tags, image_count).
- `ProductDetails` (полная карточка).
- `FilterOptions` (`brands`, `categories`, `tags`, `sizes`, `conditions`).
- `SearchFilters` (category, brand, tags, price_min/max, size, condition).

### 4.2 `product/client.py`

- `ProductClient(base_url, internal_token, timeout)` поверх `httpx.AsyncClient`.
- Методы: `get_products(filters)`, `get_product(id)`, `get_filter_options()`.
- Retry: 2 попытки на 5xx/timeout с экспоненциальным backoff.
- Circuit breaker (простой: skip на 30с после > 50% ошибок за 60с — реализуем минимально, расширим в Phase 5).
- Все запросы шлют `X-Internal-Token` для internal endpoints (для публичных `/products` — без него, либо вообще через nginx).

### 4.3 `llm/tools.py`

JSON Schema + Pydantic-валидаторы для двух tools:
- `search_products(category?, brand?, tags?, price_min?, price_max?, size?, condition?)`.
- `list_facets(field: "tags"|"brands"|"categories")`.
- На вход LLM передаём `tools=[...]` в формате OpenAI function-calling.
- На выходе — `parse_tool_call(tool_call) -> ToolCall` с Pydantic-валидацией. Если LLM прислал мусор — возвращаем `tool_result` с `{"error":"validation", "schema": {...}}`, модель повторяет.

### 4.4 `llm/loop.py`

Tool-use цикл:
```python
async def run(conversation, user_message):
    messages = build_prompt(conversation)
    for iteration in range(MAX_TOOL_ITERATIONS):
        async for chunk in llm.chat_stream(messages, tools=TOOLS):
            if chunk.is_token:           yield Event.token(chunk.text)
            elif chunk.is_tool_call:     pending_tool_calls.append(chunk)
        if not pending_tool_calls:       break
        for tc in pending_tool_calls:
            result = await dispatch_tool(tc)
            messages.append(tool_result_message(tc, result))
            if tc.name == "search_products" and result.items:
                yield Event.products([i.id for i in result.items])
    else:
        # hard-cap: финальный заход без tools
        async for chunk in llm.chat_stream(messages, tool_choice="none"):
            yield Event.token(chunk.text)
    yield Event.done()
```

### 4.5 `handlers/chat.py` — обновление

- Заменить «прямой стрим» на цикл `loop.run(...)`.
- SSE-события: `event: token`, **`event: products data: {"ids":[...]}`** (после tool, отдавшего товары), `event: error`, `event: done`.
- В `Conversation.last_product_ids` сохранять последний возвращённый список — пригодится в Phase 4 для drill-in.

### 4.6 `llm/prompts.py` — обновление

`system_prompt_v1(user_role, facets_snippet)` — добавить описание двух tools, инжект свежих фасетов в начале новой сессии (один запрос `list_facets("tags"|"brands"|"categories")` при создании conversation, кеш per-conversation).

### 4.7 Тесты

- `tests/unit/test_tool_validation.py` — невалидные args → tool error.
- `tests/integration/test_search_flow.py` — stub `product-service` через `pytest-httpserver`, OpenAI через VCR-кассету: чат «кожаные сумки до 10к» → ассистент зовёт `search_products` → SSE содержит `event: products`.
- `tests/integration/test_loop_cap.py` — если LLM зацикливается на tool_calls, после `MAX_TOOL_ITERATIONS` ассистент завершает корректным `event: done`.

### 4.8 Критерии готовности (Phase 3)

- Пример 1 из `DESIGN.md` проходит вручную через `docker compose up`.
- При невалидных tool-args (`category="bagz"`) → модель сама корректирует через `list_facets`, не падает в 5xx.
- При 5xx от `product-service` — retry, потом tool error, финальный ответ-fallback пользователю.

---

## 5. Фазы 0a/0b — что нужно от product-service до P4

P4 невозможна без векторного хранилища и hybrid SQL на стороне `product-service`. **Можно вести параллельно с P1–P3.** Здесь — короткий чек-лист от лица AI-инженера, что должно появиться у соседей; полные критерии — в `ROADMAP.md`.

### 5.1 Phase 0a (text)
- `pgvector` extension включён в product DB.
- Миграция `product_text_embeddings(product_id PK, embedding vector(1024), text_hash, updated_at)` + HNSW-индекс.
- `MultimodalEmbedder` (Rust) под Cohere, режим text.
- `--mode=text-embedder` job в product-service.
- `POST /internal/products/semantic-search` со скелетом hybrid SQL (на этой фазе `image_score = 0`).
- `X-Internal-Token` — защита эндпоинта.

### 5.2 Phase 0b (image)
- Миграция `product_image_embeddings(product_id, image_idx, embedding vector(1024), image_hash, updated_at)` + HNSW + btree по `product_id`.
- Job-kind `embed_product_images` (читает `medium.webp`, embed через Cohere image-mode, UPSERT).
- Хуки в `upload_product_images` / `delete_product_images`.
- Backfill `reindex_product_images`.
- Hybrid SQL: `MAX(1 - (embedding <=> $q))` по фото + `score_breakdown.{text,image,best_image_idx}`.
- Env: `IMAGE_EMBED_AGGREGATION`, `HYBRID_TEXT_WEIGHT`, `HYBRID_IMAGE_WEIGHT`, `IMAGE_EMBED_DAILY_CAP`.

### 5.3 Контракт — общий с QA
- `tests/contracts/semantic_search.json` живёт **в обоих сервисах** (по копии) и читается тестами обеих сторон. Источник правды — QA, BE синхронизирует.

---

## 6. Фаза 4 — semantic_search и drill-in (hybrid text + image)

**Зачем:** Примеры 2 и 3 из `DESIGN.md`. Включает image-канал в выдачу.

### 6.1 `pyproject.toml`
- Добавить `cohere>=5`.

### 6.2 `embeddings/multimodal_client.py` (новый подкаталог)
- `MultimodalEmbedder` с `embed_text(query: str) -> list[float]` (размерность = `MULTIMODAL_EMBED_DIM = 1024`).
- Под капотом — Cohere `embed-multilingual-v3.0` (`input_type="search_query"`).
- Retry x2 на 429/5xx с backoff; на ошибке — рейзит `UpstreamUnavailable`.
- Метрики: `stylist_embed_requests_total{provider,result}`, `stylist_embed_request_duration_seconds{provider}`.

### 6.3 `llm/tools.py` — расширение
- `semantic_search(query: str, top_k: int = 10, filters?)`:
  1. Лимит `QUERY_EMBED_DAILY_CAP` (in-memory счётчик, ключ — сегодняшняя дата).
  2. `vec = multimodal.embed_text(query)`.
  3. `POST /internal/products/semantic-search` с `vec`, `top_k`, `filters`, `X-Internal-Token`.
  4. Парсим `score_breakdown`, кладём в tool_result. Это дает LLM возможность сказать «подошло по описанию» / «подошло по виду».
- `get_product_details(id: UUID)` → `product_client.get_product(id)`.

### 6.4 `domain/conversation.py` — расширение
- `Conversation.last_product_ids` уже есть с Phase 3.
- При сборке prompt в `loop.run` — добавлять короткое system-сообщение `"Ранее показанные товары: [id1, id2, ...]"`, если список не пуст (нужно для drill-in примера 3).

### 6.5 `llm/prompts.py` — `system_prompt_v2`
- Описание всех 4 tools.
- Правила выбора tool (точечные критерии → `search_products`; размытое описание → `semantic_search`; «подробнее про N» → `get_product_details`).

### 6.6 Тесты
- `tests/contracts/semantic_search.json` — добавить `score_breakdown` секцию, проверить, что AI-сторона корректно парсит.
- `tests/integration/test_semantic_flow.py` — stub `MultimodalEmbedder` (детерминированный fake-вектор), stub `product-service` отдаёт фиксированный ответ со `score_breakdown`. Прогон Примеров 2 и 3.
- `tests/unit/test_query_embed_cap.py` — при достижении cap tool возвращает error, LLM получает чёткое сообщение «cap», переключается на `search_products` (проверяется prompt-snapshot).

### 6.7 Эвал (с QA)
- Совместно с QA: 30+ запросов в `eval/` (text-преобладающие / vision-преобладающие / смешанные).
- Прогоны при `HYBRID_TEXT_WEIGHT/HYBRID_IMAGE_WEIGHT` ∈ {(0.7,0.3), (0.5,0.5), (0.3,0.7)} — выбираем оптимальные дефолты, фиксируем в `product-service/.env.example`.

### 6.8 Критерии готовности (Phase 4)
- Примеры 2 и 3 проходят end-to-end.
- При выключенных embedder-ах в PS — `semantic_search` отвечает пусто, LLM объясняет «не нашёл», не падает.
- Дневной cap query-эмбеддингов реально режет (видно в логах).
- `score_breakdown` доезжает в финальный ответ LLM (видно в логах chat-handler).

---

## 7. Фаза 5 — стоимость, лимиты, прод-устойчивость

**Зачем:** перейти от «работает» к «можно открыть пользователям».

### 7.1 `ratelimit.py`
- Token bucket per user:
  - минутный — `RATE_LIMIT_MESSAGES_PER_MIN`.
  - дневной — `RATE_LIMIT_TOKENS_PER_DAY` (обновляется по фактическому usage из OpenAI response).
- При превышении — `event: error code=rate_limited` БЕЗ вызова OpenAI.

### 7.2 Circuit breaker
- Отдельные счётчики для OpenAI / Cohere / product-service.
- Порог: > 50% ошибок за 60с → break на 30с (мини-конечный автомат, без внешних либ).
- В состоянии break — сразу `event: error code=upstream_unavailable`, `event: done`.

### 7.3 Логи и метрики (`observability.py`)
- Все метрики из `DESIGN.md → Наблюдаемость` (counter/histogram/gauge — см. таблицу).
- Структурированные логи: `chat_started`, `tool_call`, `tool_result`, `chat_finished`, `rate_limited`, `upstream_error` с `request_id`, `conversation_id`, `user_id_hash` (sha256 первых 16 байт).
- Содержимое сообщений пользователя — **только** на `LOG_LEVEL=debug`.

### 7.4 Унификация error-путей
- Все возможные ошибки в `loop.run` оборачиваются в `event: error code=...` + `event: done` (никаких 500 в SSE, никаких внезапных закрытий соединения).
- `code` ∈ `rate_limited | upstream_unavailable | bad_request | internal`.

### 7.5 Конфиг
- Все таймауты/retry-параметры — через env (см. `DESIGN.md → Конфигурация`).

### 7.6 Тесты / нагрузка (с QA)
- Локальный load-тест (`hey`/`wrk` или python-скрипт) — 50 параллельных SSE-чатов 5 минут. Проверяем: нет OOM, нет гонок в `ConversationStore`, p95 < 5с.
- Сценарии деградации:
  - выключаем product-service → пользователь получает `event: error code=upstream_unavailable` < 10с (а не 30с timeout).
  - выключаем OpenAI → то же самое.
  - выключаем Cohere → `semantic_search` падает, LLM переключается на `search_products`.

### 7.7 Критерии готовности (Phase 5)
- 50 RPS минутами — сервис не падает.
- При выключенных зависимостях клиент видит корректные SSE-ошибки.
- В логах нет user content на `LOG_LEVEL=info`.

---

## 8. Фаза 6 — релиз

### 8.1 Документация
- `README.md` ai-service: build, run, env-vars, как тестировать, как смотреть метрики.
- Объявление в корневом `README.md`: новый сервис, точка входа `/api/stylist/chat`.

### 8.2 Прод-обвязка
- `assets/docker-compose.ai.prod.yml` — image из реестра, `restart: unless-stopped`, env_file, healthcheck.
- Корневой `docker-compose.prod.yml` — секция `sc-ai-service`.
- `nginx` прод-конфиг (если отличается от dev) — синхронно.
- Прод-секреты в secret store: `OPENAI_API_KEY`, `MULTIMODAL_EMBED_API_KEY`, `INTERNAL_API_TOKEN`.

### 8.3 Smoke
- `scripts/smoke-stylist.sh` — поднимает compose, прогоняет 4 примера из `DESIGN.md`, проверяет `event: done` без `event: error`. Запуск ручной (нужен `OPENAI_API_KEY`).

### 8.4 Чек-лист релиза
- Миграции PS применены, `sc-product-text-embedder` отработал ≥ 1 полный цикл, `product_image_embeddings` непустая (после первой загрузки фото).
- Дашборд метрик зелёный: `stylist_chat_messages_total` и `stylist_openai_tokens_total` живые.

---

## 9. Зависимости и риски

| Риск | Митигация |
|---|---|
| Cohere в проде окажется недоступен / дорог | `MultimodalEmbedder` — абстракция, не привязка. Смена провайдера = полная переиндексация (downtime), но не переписывание AI. Решение по результатам eval-спайка Phase 0a. |
| OpenAI rate limits на проде | `MAX_TOOL_ITERATIONS`, `MAX_TOKENS_PER_REPLY`, per-user RPS лимит, circuit breaker. |
| SSE не работает через прокси/CDN | `proxy_buffering off`, `proxy_read_timeout 600s` в nginx, `event: ping` каждые 15с в `sse-starlette`. |
| Контракт `/internal/semantic-search` разъезжается между AI и PS | `tests/contracts/semantic_search.json` — общая JSON-фикстура, тесты с обеих сторон валидируют форму. |
| In-memory история теряется при рестарте | Документировано как ограничение MVP. Post-MVP — Redis. Phase 5 покрывает graceful degradation, но не персистентность. |
| Размерность вектора (1024) зашита в миграции PS | Менять `MULTIMODAL_EMBED_DIM` после Phase 0a = пересоздание таблиц + полный reindex. Считаем зафиксированной. |

---

## 10. Что НЕ делаем в MVP

См. `DESIGN.md → Что вне MVP`. Кратко:
- Персистентная история (Redis/Postgres).
- Горизонтальное масштабирование `ai-service`.
- Re-ranking результатов.
- Image-search tool («найди по фото») — инфраструктура (joint multimodal space) будет после P0b, но сам tool откладываем.
- Distributed tracing (OTel).
- Multi-tenant rate-limit, A/B prompt-вариантов, feedback loop.

Если что-то из этого начнёт блокировать P6 — сначала правим `DESIGN.md`, потом этот план.
