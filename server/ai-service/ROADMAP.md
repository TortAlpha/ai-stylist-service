# Stylist Service — roadmap

План разбит на фазы. Каждая фаза замкнута: после завершения её можно мержить в master, и предыдущие фазы продолжают работать. Цели описаны в `DESIGN.md` — здесь только последовательность работ и критерии готовности.

Условные обозначения:
- **PS** — `server/product-service` (Rust)
- **AI** — `server/ai-service` (Python)
- **NG** — `nginx/`
- **CO** — корневой `docker-compose.yml` / `docker-compose.prod.yml`

---

## Phase 0 — pgvector в product-service (фундамент)

Без этого `semantic_search` невозможен, и Phase 4 заблокирована. Можно делать параллельно с Phase 1–3.

**Артефакты:**
- **PS** `migrations/runtime/NNNN_pgvector.sql`: `CREATE EXTENSION vector` + таблица `product_embeddings(product_id PK, embedding vector(1536), text_hash TEXT, updated_at TIMESTAMPTZ)` + `CREATE INDEX ... USING hnsw (embedding vector_cosine_ops)`.
- **PS** `src/jobs/embedder.rs` (новый job): SELECT всех `status='ready' AND is_deleted=false` товаров, для каждого считает `text_hash` от `generate_product_text(id)`, пропускает несменившиеся, батчем зовёт OpenAI embeddings, делает UPSERT.
- **PS** запуск: режим `--mode=embedder` у того же бинарника + переменные `OPENAI_API_KEY`, `OPENAI_EMBED_MODEL`, `EMBEDDER_BATCH_SIZE` (default 64), `EMBEDDER_INTERVAL_SECONDS` (default 600), `EMBEDDER_DAILY_CAP`.
- **PS** `POST /internal/products/semantic-search` (handler + repo + контракт из DESIGN.md). Защита `X-Internal-Token`. В nginx наружу не проксируется.
- **PS** unit-тесты на `text_hash` стабильность и на исключение `is_deleted/!ready` из выдачи.
- **CO** добавить отдельный сервис `sc-product-embedder` (тот же image, другая команда) либо включить cron/job в основной `sc-product-service`. На MVP — отдельный сервис, легче выключить.
- **PS** `INTERNAL_API_TOKEN` в `.env.example`.

**Готово, когда:**
- Миграция применяется на чистой и существующей базе без ошибок.
- `sc-product-embedder` за один проход индексирует ≥ 99% `ready`-товаров; повторный проход не делает лишних OpenAI-вызовов (`text_hash` режет).
- `POST /internal/products/semantic-search` возвращает top-10 за < 100ms p95 на тестовом наборе из 1000 товаров.
- Неверный `X-Internal-Token` → 401.
- `EMBEDDER_DAILY_CAP` реально режет: при достижении воркер логирует и спит до следующих суток.

---

## Phase 1 — каркас ai-service (без LLM)

Минимальный сервис, который поднимается, проксируется через nginx и проходит auth.

**Артефакты:**
- **AI** `pyproject.toml` (fastapi, uvicorn[standard], openai, httpx, pydantic, pydantic-settings, sse-starlette, structlog, prometheus_client, tiktoken; dev: pytest, pytest-asyncio, pytest-httpserver, ruff, mypy).
- **AI** `Dockerfile` (multi-stage, неприв. пользователь, `CMD uvicorn ai_service.main:app --port 8084`).
- **AI** `assets/docker-compose.ai.yml` + `docker-compose.ai.prod.yml` (по образцу product-service).
- **AI** `src/ai_service/`: `main.py` (FastAPI app + lifespan + `/healthz` + `/metrics`), `config.py` (pydantic-settings), `auth.py` (зависимость `current_user` из `X-User-Id`/`X-User-Role`), `observability.py` (structlog setup, request_id middleware).
- **AI** один эндпоинт-заглушка `GET /api/stylist/whoami` → возвращает `user_id` из заголовков (нужен для проверки auth).
- **AI** базовые тесты: `/healthz` отвечает 200; `/api/stylist/whoami` без `X-User-Id` → 401; с заголовками → 200 c id.
- **NG** в `nginx.conf` добавить `upstream ai-service` и `location /api/stylist/` с `auth_request`, `proxy_buffering off`, `proxy_read_timeout 600s` (см. DESIGN.md → Деплой).
- **CO** добавить `sc-ai-service` в корневой `docker-compose.yml`, `depends_on` на `sc-auth-service` (healthy) и `sc-product-service` (healthy). Расширить `nginx.depends_on` сервисом `sc-ai-service`.
- **AI** `README.md` с парой команд: build, up, env-vars (минимум).

**Готово, когда:**
- `docker compose up` поднимает всё, `curl http://localhost/api/stylist/whoami` без JWT → 401, с валидным JWT → 200.
- `/metrics` отдаёт прометеусные текстовые метрики.
- `/healthz` не зависит от OpenAI / product-service.

---

## Phase 2 — chat без tools (SSE, без подбора товаров)

Вертикальный срез: пользователь говорит → модель отвечает потоком, история помнится, lifecycle сессии работает. Tool-use ещё нет.

**Артефакты:**
- **AI** `domain/conversation.py` — `Conversation`, `Message`, in-memory `ConversationStore` с TTL и cap по сообщениям.
- **AI** `llm/client.py` — обёртка над OpenAI SDK (chat.completions.create stream, retry на 429/5xx).
- **AI** `llm/prompts.py` — системный промпт v0 (без описания tools).
- **AI** `llm/budget.py` — обрезка истории по `HISTORY_TOKEN_BUDGET` через `tiktoken`.
- **AI** `handlers/chat.py` — `POST /api/stylist/chat` SSE: stream `event: token`, в конце `event: done`. Heartbeat `event: ping` каждые 15с (через `sse-starlette.EventSourceResponse(ping=15)`).
- **AI** `handlers/conversations.py` — `POST` (создать), `DELETE /{id}` (очистить).
- **AI** unit-тесты на budget trim и conversation TTL; integration-тест с замоканным OpenAI: новый conv → отправить сообщение → получить хотя бы один `event: token` и финальный `event: done`.
- **AI** Smoke: вручную чат «привет» → ответ.

**Готово, когда:**
- В реальном браузере (через `fetch` с `text/event-stream` парсером или curl) виден поток токенов.
- При перезапуске контейнера история ожидаемо пропадает (это ок для MVP — задокументировано).
- Срабатывает обрезка истории: после 30+ сообщений старые сворачиваются (вручную проверяется по логам, что префикс заменён `[summary]`).

---

## Phase 3 — точечный поиск (search_products + list_facets)

Первый полезный сценарий: пример 1 из DESIGN.md.

**Артефакты:**
- **AI** `product/client.py` — async httpx-клиент к `product-service`: `get_products(filters)`, `get_product(id)`, `get_filter_options()`. Таймауты, экспоненциальный backoff на 5xx/timeout, circuit breaker.
- **AI** `llm/tools.py` — JSON Schema для `search_products` и `list_facets`. Pydantic-модели для парсинга ответа LLM в tool_call.
- **AI** `llm/loop.py` — tool-use цикл: chat stream → если есть `tool_calls` → выполнить → подложить tool_result → повторить, hard-cap `MAX_TOOL_ITERATIONS`.
- **AI** SSE-событие `event: products data: {"ids": [...]}` после tool-результата с товарами.
- **AI** обновление системного промпта: добавить описание двух tools.
- **AI** integration-тест: stub product-service → чат «кожаные сумки до 10к» → ассистент должен позвать `search_products` и в SSE прийти `event: products`.

**Готово, когда:**
- Пример 1 из DESIGN.md проходит вручную в реальном compose.
- При невалидных tool-args (`category="bagz"`) tool возвращает error, модель сама корректирует и переходит к `list_facets` либо сдаётся в ≤ `MAX_TOOL_ITERATIONS` шагов — без 5xx из ai-service.
- product-service-клиент при 5xx делает retry, при стабильном падении даёт LLM tool error и финальный ответ-fallback приходит пользователю.

---

## Phase 4 — semantic_search и drill-in

Размытые запросы и контекстные follow-up. Зависит от Phase 0.

**Артефакты:**
- **AI** `llm/tools.py` — добавить `semantic_search` (берёт query string, считает embedding через OpenAI, кладёт vector в `POST /internal/products/semantic-search`) и `get_product_details`.
- **AI** в `Conversation` — поле `last_product_ids` (обновляется после каждого ответа с товарами) + инжект в системный контекст следующего хода `"Ранее показанные товары: [...]"`.
- **AI** `EMBEDDINGS_DAILY_CAP` — счётчик in-memory; при упоре tool возвращает error, LLM переключается на `search_products`.
- **AI** контрактный тест `tests/contracts/semantic_search.json`, читаемый и в AI, и в PS-тестах.
- **AI** integration-тесты на примеры 2 и 3 (с замоканным OpenAI).
- Системный промпт обновлён под все 4 tools.

**Готово, когда:**
- Примеры 2 и 3 из DESIGN.md проходят end-to-end.
- При выключенном `sc-product-embedder` semantic-search возвращает пусто → LLM это объясняет, не падает.
- Дневной cap эмбеддингов реально срезает (логируется hit).

---

## Phase 5 — стоимость, лимиты, прод-устойчивость

Пройти от «работает» к «можно открывать в прод».

**Артефакты:**
- **AI** `ratelimit.py` — token bucket per user (минутный — сообщения, дневной — токены). При превышении `event: error code=rate_limited` без вызова OpenAI.
- **AI** все error-пути приводят к `event: error` + `event: done` (а не разрыв соединения).
- **AI** structlog-логи на каждое ключевое событие (`chat_started`, `tool_call`, `tool_result`, `chat_finished`, `rate_limited`, `upstream_error`); user content только на `LOG_LEVEL=debug`.
- **AI** все метрики из DESIGN.md → Наблюдаемость подняты.
- **AI** circuit breaker на OpenAI и product-service (порог: > 50% ошибок за 60с → break на 30с).
- **AI** load-тест локально (`hey`/`wrk` или python-скрипт): 50 параллельных SSE-чатов в течение 5 минут — отсутствие OOM, отсутствие гонок в conversation store.
- **AI** конфиг таймаутов и retry-параметров вынесен в env (см. таблицу в DESIGN.md → Конфигурация).

**Готово, когда:**
- Под нагрузкой 50 RPS (минутными сериями) сервис не падает, метрики `stylist_chat_duration_seconds` p95 < 5с при cached prompts.
- При выключенном product-service чат отвечает «каталог временно недоступен» в течение < 10с (а не висит 30с timeout-ом).
- При выключенном OpenAI клиент видит `event: error code=upstream_unavailable` и нормальный `event: done`.
- В логах нет user content на `LOG_LEVEL=info`.

---

## Phase 6 — релиз

Подготовка к мерджу в прод и публикации.

**Артефакты:**
- **AI** `README.md` с инструкцией: build, run, env-vars, как тестировать, как смотреть метрики.
- **AI** `assets/docker-compose.ai.prod.yml` синхронизирован с `docker-compose.prod.yml`.
- **NG** прод-конфиг nginx (если он отличается от dev) обновлён симметрично.
- **CO** в корневой `docker-compose.prod.yml` добавлен `sc-ai-service` (image из реестра).
- **AI** `.env.example` обновлён (см. таблицу в DESIGN.md).
- **AI** smoke-скрипт `scripts/smoke-stylist.sh` — поднять compose, прогнать 4 примера, проверить `event: done`. Запуск ручной (нужен OPENAI_API_KEY).
- Чек-лист релиза: миграции применены, `sc-product-embedder` уже отработал хотя бы один полный цикл, `OPENAI_API_KEY` и `INTERNAL_API_TOKEN` положены в прод-секреты.
- Объявление в README корня репо: новый сервис, точка входа `/api/stylist/chat`.

**Готово, когда:**
- На проде доступен `/api/stylist/chat`, smoke-тест проходит зелёным.
- Дашборд метрик показывает живые `stylist_chat_messages_total` и `stylist_openai_tokens_total`.

---

## Post-MVP (Phase 7+)

В порядке убывания пользы / возрастания сложности — не закладываем дату, делаем по необходимости:

1. **Persist conversation history** в Redis (или Postgres). Снимает риск потери истории при рестарте, разблокирует горизонтальное масштабирование `stylist-service`.
2. **Push-обновление product embeddings** через шину событий из product write flow вместо периодического polling-а. Уменьшает задержку «новый товар → попал в semantic_search».
3. **Re-ranking** результатов `semantic_search` cross-encoder-ом (или второй LLM-проверкой) для топ-20 → топ-5. Замерять offline на 30-запросном наборе.
4. **Персонализация:** учёт истории просмотров и покупок пользователя при формировании query / фильтров (нужно расширение `user-service` API).
5. **Метрики качества рекомендаций:** связать SSE `event: products` с последующими `add-to-cart` / `view` событиями, считать CTR per session. Требует event-pipeline за пределами ai-service.
6. **Distributed tracing (OpenTelemetry)**: единый trace через nginx → ai-service → product-service → OpenAI.
7. **Feedback loop:** thumbs up/down на ответе → лог в evaluation датасет для прогона на новых моделях.
8. **A/B-тестирование** prompt-вариантов и моделей по сегментам пользователей.
9. **Multi-tenant rate-limit** (по плану/роли).
10. **Кеш ответов** на типовые запросы («что-то на лето», «бюджет до 5к») с TTL ~ часы.

---

## Зависимости между фазами

```
Phase 0 ──┐
          ├──► Phase 4
Phase 1 ──┴──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6
```

Phase 0 и Phase 1 можно вести параллельно. Phase 2 и Phase 3 — строго после Phase 1. Phase 4 ждёт обе ветки (0 и 3). Phase 5 и 6 — последовательно.

## Не делаем в MVP

Список вне MVP — в DESIGN.md → «Что вне MVP». Если что-то из этого блокирует Phase 6 (например, выясняется, что без Redis прод не выдержит) — пересматриваем DESIGN.md перед тем, как двигать ROADMAP.
