# Stylist Service — roadmap

План разбит на фазы. Каждая фаза замкнута: после завершения её можно мержить в master, и предыдущие фазы продолжают работать. Цели описаны в `DESIGN.md` — здесь только последовательность работ и критерии готовности.

Условные обозначения:
- **PS** — `server/product-service` (Rust)
- **AI** — `server/ai-service` (Python)
- **NG** — `nginx/`
- **CO** — корневой `docker-compose.yml` / `docker-compose.prod.yml`

---

## Phase 0a — pgvector + text embeddings в product-service

Фундамент под `semantic_search`. Phase 4 без него заблокирована (можно делать параллельно с Phase 1–3). Multimodal-провайдер уже зафиксирован (Cohere `embed-multilingual-v3.0`, dim 1024) — он общий и для text-, и для image-стороны (вариант B, single multimodal space).

**Артефакты:**
- **Провайдер зафиксирован** до начала фазы: Cohere `embed-multilingual-v3.0`, dim 1024 (см. DESIGN.md → «Открытые вопросы»). В `.env.example` обоих сервисов проставить `MULTIMODAL_EMBED_PROVIDER=cohere`, `MULTIMODAL_EMBED_MODEL=embed-multilingual-v3.0`, `MULTIMODAL_EMBED_DIM=1024`. Eval-спайк на 30-запросном датасете остаётся — но не для выбора провайдера, а для подтверждения recall@10 на русском каталоге и стартовых значений `HYBRID_TEXT_WEIGHT` / `HYBRID_IMAGE_WEIGHT`.
- **PS** `migrations/runtime/NNNN_pgvector_text.sql`: `CREATE EXTENSION vector` + таблица `product_text_embeddings(product_id PK REFERENCES product(id) ON DELETE CASCADE, embedding vector(:dim), text_hash TEXT, updated_at TIMESTAMPTZ)` + `CREATE INDEX ... USING hnsw (embedding vector_cosine_ops)`. `:dim` подставляется из `MULTIMODAL_EMBED_DIM` на момент применения.
- **PS** `src/embeddings/multimodal_client.rs` (новый): абстракция `MultimodalEmbedder` с `embed_text(&[String]) -> Vec<Vec<f32>>` и `embed_image(&[Bytes]) -> Vec<Vec<f32>>`. Реализация под Cohere `embed-multilingual-v3.0` (REST, base64 для картинок), плюс mock для тестов. Абстракция нужна на случай смены провайдера post-MVP — но это разовая операция с полной переиндексацией, не runtime-переключение.
- **PS** `src/jobs/text_embedder.rs` (новый): SELECT всех `status='ready' AND is_deleted=false` товаров, для каждого считает `text_hash` от `generate_product_text(id)`, пропускает несменившиеся, батчем зовёт `MultimodalEmbedder.embed_text`, UPSERT в `product_text_embeddings`.
- **PS** запуск: режим `--mode=text-embedder` у того же бинарника + переменные `MULTIMODAL_EMBED_*`, `TEXT_EMBEDDER_BATCH_SIZE` (default 64), `TEXT_EMBEDDER_INTERVAL_SECONDS` (default 600), `TEXT_EMBEDDER_DAILY_CAP`.
- **PS** `POST /internal/products/semantic-search` (handler + repo + контракт из DESIGN.md). На этой фазе `image_score = 0` (image-таблицы ещё нет), `score = HYBRID_TEXT_WEIGHT * text_cos`. Защита `X-Internal-Token`. В nginx наружу не проксируется.
- **PS** unit-тесты на `text_hash` стабильность и на исключение `is_deleted/!ready` из выдачи.
- **CO** добавить отдельный сервис `sc-product-text-embedder` (тот же image, другая команда). На MVP — отдельный сервис, легче выключить.
- **PS** `INTERNAL_API_TOKEN`, `MULTIMODAL_EMBED_*` в `.env.example`.

**Готово, когда:**
- Миграция применяется на чистой и существующей базе без ошибок при подставленной `MULTIMODAL_EMBED_DIM`.
- `sc-product-text-embedder` за один проход индексирует ≥ 99% `ready`-товаров; повторный проход не делает лишних embed-вызовов (`text_hash` режет).
- `POST /internal/products/semantic-search` возвращает top-10 за < 100ms p95 на тестовом наборе из 1000 товаров (с пустой image-таблицей).
- Неверный `X-Internal-Token` → 401.
- `TEXT_EMBEDDER_DAILY_CAP` реально режет: при достижении воркер логирует и спит до следующих суток.

---

## Phase 0b — image embeddings в product-service

Включает image-канал в hybrid score. Phase 4 ждёт обе подфазы (0a и 0b). Можно делать параллельно с Phase 2–3 после завершения 0a.

**Артефакты:**
- **PS** `migrations/runtime/NNNN_pgvector_images.sql`: таблица `product_image_embeddings(product_id, image_idx, embedding vector(:dim), image_hash TEXT, updated_at TIMESTAMPTZ, PRIMARY KEY (product_id, image_idx), FOREIGN KEY (product_id) REFERENCES product(id) ON DELETE CASCADE)` + HNSW-индекс по `embedding` + btree по `product_id`.
- **PS** новый job-kind `embed_product_images` в `src/jobs/`: payload `{ product_id, image_indices: [...] }`. Handler читает `medium.webp` каждого индекса из S3, считает `image_hash` от bytes, пропускает несменившиеся, зовёт `MultimodalEmbedder.embed_image`, UPSERT в `product_image_embeddings`.
- **PS** хук в `upload_product_images`: после успешного `put_object` всех вариантов — `enqueue_embed_product_images` в той же транзакции (атомарно с инкрементом `image_count`).
- **PS** хук в `delete_product_images`: после удаления из S3 — `DELETE FROM product_image_embeddings WHERE product_id=$1 AND image_idx = ANY($2)` (или по `product_id` для cascade при soft-delete продукта).
- **PS** single-shot backfill-job `reindex_product_images`: перебирает `ready` товары, у которых строк в `product_image_embeddings` меньше, чем `image_count`, enqueue-ит `embed_product_images` для недостающих индексов.
- **PS** расширить hybrid SQL в `/internal/products/semantic-search`: добавить ветку `image_scores` с `MAX(1 - (embedding <=> $q))`, итоговый score — `HYBRID_TEXT_WEIGHT * text_score + HYBRID_IMAGE_WEIGHT * image_score`. Ответ дополнить `score_breakdown.{text,image,best_image_idx}`.
- **PS** env-флаги: `IMAGE_EMBED_AGGREGATION=per_image|centroid` (default `per_image`), `HYBRID_TEXT_WEIGHT=0.5`, `HYBRID_IMAGE_WEIGHT=0.5`, `IMAGE_EMBED_DAILY_CAP`. В `.env.example`.
- **PS** unit-тесты: `image_hash` стабильность; `embed_product_images` пропускает несменившиеся; `delete_product_images` чистит и таблицу эмбеддингов; hybrid SQL корректно работает при отсутствии у товара text- или image-вектора (через `COALESCE`).
- **PS** контрактный тест на новый формат ответа `score_breakdown` (читается и AI-стороной).

**Готово, когда:**
- Миграция применяется без ошибок.
- Загрузка нового товара с фото → через минуту строки появляются в `product_image_embeddings` без ручных шагов.
- Удаление фото / товара → строки исчезают.
- `reindex_product_images` за один проход догоняет ≥ 99% существующих `ready`-товаров.
- `/internal/products/semantic-search` p95 < 150ms на наборе из 1000 товаров со средним `image_count=4` в `per_image` режиме.
- При выключенном image-embedder (и пустой image-таблице) endpoint не падает, возвращает только text-score.
- `IMAGE_EMBED_DAILY_CAP` реально режет.

---

## Phase 1 — каркас ai-service (без LLM)

Минимальный сервис, который поднимается, проксируется через nginx и проходит auth.

**Артефакты:**
- **AI** `pyproject.toml` (fastapi, uvicorn[standard], openai, httpx, pydantic, pydantic-settings, sse-starlette, structlog, prometheus_client, tiktoken; `cohere` SDK добавляется в Phase 4 вместе с `MultimodalEmbedder`-клиентом; dev: pytest, pytest-asyncio, pytest-httpserver, ruff, mypy).
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

## Phase 4 — semantic_search и drill-in (hybrid text + image)

Размытые запросы и контекстные follow-up. Зависит от Phase 0a **и** 0b.

**Артефакты:**
- **AI** `embeddings/multimodal_client.py` — обёртка над Cohere `embed-multilingual-v3.0` с `embed_text(query)`. Используется одна и та же модель и dim (1024), что и в `product-service` — иначе query окажется в чужом пространстве.
- **AI** `llm/tools.py` — добавить `semantic_search` (берёт query string, считает embedding через `multimodal_client`, кладёт vector в `POST /internal/products/semantic-search`) и `get_product_details`.
- **AI** в `Conversation` — поле `last_product_ids` (обновляется после каждого ответа с товарами) + инжект в системный контекст следующего хода `"Ранее показанные товары: [...]"`.
- **AI** `QUERY_EMBED_DAILY_CAP` — счётчик in-memory; при упоре tool возвращает error, LLM переключается на `search_products`.
- **AI** парсинг `score_breakdown` из ответа `/internal/semantic-search` — прокинуть в tool_result так, чтобы LLM мог аргументировать («подошло по описанию», «подошло по виду»). Метрика `stylist_semantic_search_score_breakdown` (histogram, лейблы `kind=text|image`).
- **AI/QA** контрактный тест `tests/contracts/semantic_search.json` — общий с PS, теперь включает `score_breakdown`.
- **AI/QA** integration-тесты на примеры 2 и 3 (с замоканным OpenAI и stub `MultimodalEmbedder`).
- **QA** eval-датасет 30+ запросов: 10 text-преобладающих («бренд X, размер M»), 10 vision-преобладающих («что-то с леопардовым принтом», «жёлтый минималистичный»), 10 смешанных. Ручная разметка релевантных id. Прогоны при разных `HYBRID_TEXT_WEIGHT` / `HYBRID_IMAGE_WEIGHT` для тюнинга.
- **PS** (если по результатам тюнинга) — обновить дефолты `HYBRID_*_WEIGHT` в `.env.example`.
- Системный промпт обновлён под все 4 tools.

**Готово, когда:**
- Примеры 2 и 3 из DESIGN.md проходят end-to-end.
- На eval-датасете hybrid-режим даёт recall@10 не хуже text-only (Phase 0a) на text-преобладающих запросах и значимо лучше на vision-преобладающих.
- При выключенных embedder-ах semantic-search возвращает пусто → LLM это объясняет, не падает.
- Дневной cap query-эмбеддингов реально срезает (логируется hit).
- `score_breakdown` доезжает до LLM и используется в финальной формулировке ответа (видно вручную в логах chat-handler).

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
2. **Push-обновление product embeddings** через шину событий из product write flow вместо периодического polling-а. Уменьшает задержку «новый товар → попал в semantic_search». (Image-сторона уже event-driven через job-queue после Phase 0b — это касается text-стороны.)
3. **Image-search tool: «найди по фото».** Пользователь грузит картинку в чате → `image_search(image_bytes, top_k)` → `multimodal_client.embed_image(bytes)` → в тот же `/internal/semantic-search` (vector совместим с image-таблицей благодаря варианту B). Вся инфраструктура уже есть, нужен только новый tool, мультипарт-аплоад на фронте и отдельный rate-limit на image-query.
4. **Re-ranking** результатов `semantic_search` cross-encoder-ом (или второй LLM-проверкой) для топ-20 → топ-5. Замерять offline на 30-запросном наборе.
5. **Centroid-режим под нагрузкой.** Если индекс `product_image_embeddings` в `per_image` режиме станет узким местом по памяти/latency — переключение на `IMAGE_EMBED_AGGREGATION=centroid` + миграция (`UPDATE` агрегатов, удаление старых строк). Решение по метрикам.
6. **Персонализация:** учёт истории просмотров и покупок пользователя при формировании query / фильтров (нужно расширение `user-service` API).
7. **Метрики качества рекомендаций:** связать SSE `event: products` с последующими `add-to-cart` / `view` событиями, считать CTR per session. Требует event-pipeline за пределами ai-service.
8. **Distributed tracing (OpenTelemetry)**: единый trace через nginx → ai-service → product-service → OpenAI / multimodal embed.
9. **Feedback loop:** thumbs up/down на ответе → лог в evaluation датасет для прогона на новых моделях.
10. **A/B-тестирование** prompt-вариантов, hybrid-весов и моделей по сегментам пользователей.
11. **Multi-tenant rate-limit** (по плану/роли).
12. **Кеш ответов** на типовые запросы («что-то на лето», «бюджет до 5к») с TTL ~ часы.

---

## Зависимости между фазами

```
Phase 0a ──► Phase 0b ──┐
                        ├──► Phase 4
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6
```

Phase 0a и Phase 1 можно вести параллельно. Phase 0b — строго после 0a (нужен `MultimodalEmbedder`-клиент и зафиксированный провайдер). Phase 2 и Phase 3 — строго после Phase 1. Phase 4 ждёт обе ветки (0a+0b и 3). Phase 5 и 6 — последовательно.

## Не делаем в MVP

Список вне MVP — в DESIGN.md → «Что вне MVP». Если что-то из этого блокирует Phase 6 (например, выясняется, что без Redis прод не выдержит) — пересматриваем DESIGN.md перед тем, как двигать ROADMAP.
