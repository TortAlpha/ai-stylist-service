# Stylist Service — дизайн

Сервис-стилист для покупателей: помогает находить товары на естественном языке, рекомендует на основе тегов, категорий, типизированных details и описаний.

## Стек

- **Python 3.12 + FastAPI + Uvicorn (uvloop)** — async, чтобы дёшево держать длинные SSE-стримы.
- **OpenAI Python SDK ≥ 1.30** — `chat.completions.create(stream=True)` с tool use. OpenAI используется **только** для chat-completion.
  - Chat-модель по умолчанию: `gpt-4o-mini` (дешёвая для tool-use loop), эскалация на `gpt-4o` через env при необходимости.
- **Multimodal embedding-модель** — единое векторное пространство для текста и изображений товаров. Той же моделью считаются: (а) text-вектор товара из `generate_product_text`, (б) image-вектор каждого фото товара (по `medium.webp`), (в) query-вектор пользовательского сообщения в `ai-service`. Провайдер фиксируется в Phase 0b после замера качества/стоимости — кандидаты: Cohere Embed-3 multimodal, Voyage `voyage-multimodal-3`, Vertex `multimodalembedding@001`, локальный open_clip / SigLIP. Размерность (`MULTIMODAL_EMBED_DIM`) — параметр выбранной модели; pgvector-таблицы создаются под фиксированную dim. OpenAI text-embedding для query/индексации **не используется** — это ломало бы общее пространство с image-векторами.
- **httpx (async)** — клиент к `product-service`: таймауты, exponential backoff, connection pool.
- **sse-starlette** — SSE-стрим в браузер с keep-alive ping.
- **Pydantic v2** — валидация tool-args от LLM и схем входящих/исходящих сообщений.
- **structlog + prometheus_client** — JSON-логи и метрики на `/metrics`.
- **Авторизация: JWT терминируется на nginx** (`auth_request` к `auth-service`, как у остальных сервисов). `stylist-service` читает уже валидные `X-User-Id` и `X-User-Role` из заголовков. Локальный JWT verify — только опционально, как defence-in-depth.

Платформа Python — отдельная от Rust-стека (auth/product/user), потому что вся OpenAI-экосистема (SDK, инструменты для prompt engineering, eval-фреймворки) живёт в Python и Node, и переписывать tool-use loop на Rust — не оправданная цена для MVP.

## Структура каталога

```
server/ai-service/
├── pyproject.toml               # uv / pip деп
├── Dockerfile
├── assets/
│   ├── docker-compose.ai.yml        # dev
│   └── docker-compose.ai.prod.yml   # prod
├── migrations/                  # пусто на MVP (state in-memory), задел на будущее
├── src/ai_service/
│   ├── __init__.py
│   ├── main.py                  # FastAPI app, роутеры, lifespan
│   ├── config.py                # pydantic-settings, чтение env
│   ├── deps.py                  # FastAPI dependencies: current_user, http clients
│   ├── auth.py                  # парсинг X-User-Id / X-User-Role
│   ├── domain/
│   │   ├── conversation.py      # Conversation, Message, ToolCall
│   │   └── errors.py
│   ├── llm/
│   │   ├── client.py            # обёртка над OpenAI SDK + retry
│   │   ├── tools.py             # JSON Schema tool definitions
│   │   ├── prompts.py           # системный промпт, шаблоны
│   │   ├── budget.py            # обрезка истории, summary-сжатие
│   │   └── loop.py              # tool-use цикл: chat -> tool_call -> tool_result -> ...
│   ├── product/
│   │   ├── client.py            # httpx-клиент к product-service
│   │   └── schemas.py           # типы DTO для product-service
│   ├── handlers/
│   │   ├── chat.py              # POST /api/stylist/chat (SSE)
│   │   └── conversations.py     # POST/DELETE /api/stylist/conversations
│   ├── ratelimit.py             # token bucket in-memory
│   └── observability.py         # logging, metrics, request_id middleware
└── tests/
    ├── conftest.py
    ├── unit/
    └── integration/
```

## Конфигурация

Все параметры — через env. Дефолты безопасные для dev, продовые секреты — из `.env` рядом с docker-compose.

| Var | Default | Назначение |
|---|---|---|
| `SERVICE_PORT` | `8084` | HTTP-порт |
| `OPENAI_API_KEY` | — | Обязательно (только chat) |
| `OPENAI_CHAT_MODEL` | `gpt-4o-mini` | Можно переопределить на `gpt-4o` |
| `OPENAI_REQUEST_TIMEOUT_SECONDS` | `30` | Таймаут одного OpenAI chat-вызова |
| `MULTIMODAL_EMBED_PROVIDER` | — | Один из: `cohere`, `voyage`, `vertex`, `local_clip`. Фиксируется в Phase 0b. Должен совпадать с `product-service` |
| `MULTIMODAL_EMBED_MODEL` | — | Имя модели у выбранного провайдера (например, `embed-multilingual-v3.0` для Cohere) |
| `MULTIMODAL_EMBED_DIM` | — | Размерность вектора. Должна совпадать с dim таблиц `product_text_embeddings` / `product_image_embeddings` |
| `MULTIMODAL_EMBED_API_KEY` | — | Секрет провайдера (для `local_clip` — пусто, модель грузится в процесс) |
| `MULTIMODAL_EMBED_BASE_URL` | — | Опционально, для self-hosted эндпоинтов |
| `MULTIMODAL_EMBED_TIMEOUT_SECONDS` | `15` | Таймаут одного embed-вызова |
| `PRODUCT_SERVICE_URL` | `http://sc-product-service:8081` | |
| `PRODUCT_SERVICE_TIMEOUT_SECONDS` | `5` | |
| `INTERNAL_API_TOKEN` | — | Shared secret для internal endpoint-ов product-service |
| `MAX_HISTORY_MESSAGES` | `30` | Hard cap до summary-сжатия |
| `HISTORY_TOKEN_BUDGET` | `6000` | Бюджет истории в промпте |
| `MAX_TOOL_ITERATIONS` | `4` | Защита от tool-use зацикливания |
| `MAX_TOKENS_PER_REPLY` | `800` | `max_tokens` в chat completion |
| `RATE_LIMIT_MESSAGES_PER_MIN` | `12` | Per user |
| `RATE_LIMIT_TOKENS_PER_DAY` | `50000` | Per user |
| `QUERY_EMBED_DAILY_CAP` | `10000` | Per service (всего query-эмбеддингов через multimodal-провайдера в сутки) |
| `CONVERSATION_TTL_MINUTES` | `30` | TTL in-memory сессии |
| `LOG_LEVEL` | `info` | |
| `JWT_SECRET` | — | Опционально, для локального verify |

Веса гибридного скоринга (`HYBRID_TEXT_WEIGHT`, `HYBRID_IMAGE_WEIGHT`) и режим агрегации картинок (`IMAGE_EMBED_AGGREGATION = per_image | centroid`) живут на стороне `product-service` (см. «Хранилища»), потому что финальный score считается там в SQL.

## Границы сервиса

```
                      +--------------------------------+
                      | stylist-service (FastAPI :8084) |
                      |                                |
 /api/stylist/chat -> |  - chat handler (SSE)          |
                      |  - conversations               |
                      +---------------+----------------+
                                      |
                    +---------------+--------+----------+----------+
                    |                        |          |          |
                    v                        v          v          v
             product-service             OpenAI API   Multimodal   auth-service
             (HTTP, source of truth,     (chat only)  Embed API    (JWT verify
              filters, pgvector)                      (query embed) через nginx)
```

JWT валидируется на nginx через `auth_request → /internal/auth/verify` (так же, как для product/user-service). Внутрь `stylist-service` приходят уже подтверждённые `X-User-Id` и `X-User-Role`. История диалогов и rate-limit-счётчики — **in-memory** в процессе `stylist-service`.

## Хранилища

### product-service / product DB

Товары, фильтры, фасеты, тексты для эмбеддинга и сами product embeddings принадлежат `product-service`.
Это сохраняет одну точку правды для статусов, soft-delete, размеров, категорий, цен и правил видимости витрины.

`stylist-service` **не** имеет прямого доступа к product DB и не хранит копии эмбеддингов.

#### Multimodal-индекс: две таблицы

Поскольку у товара есть текстовое представление и **N фото**, индекс разбит на две таблицы. Обе считаются одной и той же multimodal-моделью (вариант B — единое векторное пространство), поэтому query-вектор сравним с обоими видами строк.

```sql
CREATE EXTENSION IF NOT EXISTS vector;

-- Текстовый вектор товара: один на товар.
-- Источник — generate_product_text(id), считаем multimodal-моделью в text-mode.
CREATE TABLE product_text_embeddings (
    product_id  UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    embedding   vector(:dim) NOT NULL,
    text_hash   TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX product_text_embeddings_hnsw
    ON product_text_embeddings
    USING hnsw (embedding vector_cosine_ops);

-- Image-вектор каждого фото товара: до N строк на товар.
-- image_idx совпадает с индексом каталога S3 `products/{product_id}/{image_idx}/`.
-- Источник — `medium.webp` варианта (компромисс качество/размер для embedder API).
CREATE TABLE product_image_embeddings (
    product_id  UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    image_idx   INT  NOT NULL,
    embedding   vector(:dim) NOT NULL,
    image_hash  TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, image_idx)
);

CREATE INDEX product_image_embeddings_hnsw
    ON product_image_embeddings
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX product_image_embeddings_product
    ON product_image_embeddings (product_id);
```

`:dim` — `MULTIMODAL_EMBED_DIM` выбранной модели (1024/1408/…); миграция параметризуется этой переменной.

`text_hash` — SHA256 от строки `generate_product_text(id)`. `image_hash` — SHA256 от байт `medium.webp` соответствующего варианта в S3 (стабильнее raw — не зависит от формата исходника). Если хэш не изменился — пересчёт пропускаем.

Текст строится в `product-service` из продуктовой модели — helper `generate_product_text(p_product_id)` уже есть и собирает name, brand, category, tags, season, details, `ai_notes`. Для image-вектора берём `medium.webp` (max edge 800px) — этого достаточно большинству API-провайдеров (Cohere/Voyage/Vertex принимают base64 / URL картинок такого размера).

#### Hybrid scoring (`per_image` режим)

Запрос-вектор сравнивается и с text-, и с image-эмбеддингами товара. Для image берётся **MAX** косинусной близости по фото — чтобы товар, у которого хотя бы один ракурс релевантен запросу, попал в выдачу. Финальный score — взвешенная сумма:

```sql
WITH text_scores AS (
    SELECT product_id, 1 - (embedding <=> $q) AS s
    FROM product_text_embeddings
),
image_scores AS (
    SELECT product_id, MAX(1 - (embedding <=> $q)) AS s
    FROM product_image_embeddings
    GROUP BY product_id
)
SELECT
    p.id,
    COALESCE(t.s, 0) AS text_score,
    COALESCE(i.s, 0) AS image_score,
    $w_text  * COALESCE(t.s, 0) +
    $w_image * COALESCE(i.s, 0) AS score
FROM product p
LEFT JOIN text_scores  t ON t.product_id = p.id
LEFT JOIN image_scores i ON i.product_id = p.id
WHERE p.status = 'ready' AND p.is_deleted = false
  AND ($category    IS NULL OR p.category_id = $category)
  AND ($price_max   IS NULL OR p.purchase_price <= $price_max)
  /* + остальные фильтры из request body */
ORDER BY score DESC
LIMIT $top_k;
```

`$w_text`, `$w_image` — `HYBRID_TEXT_WEIGHT` и `HYBRID_IMAGE_WEIGHT` из env product-service (default 0.5 / 0.5; тюнятся на eval-датасете в Phase 4). Веса нормировать не обязательно — итоговый score интерпретируется относительно других кандидатов в той же выдаче, не как абсолютная вероятность.

`COALESCE(..., 0)` страхует случай «у товара ещё нет text- или image-вектора» (новый товар, эмбеддер не успел). В Phase 4 в логи прокидываем долю товаров без вектора, чтобы видеть отставание embedder-а.

#### Альтернативный режим: `centroid`

Под env-флагом `IMAGE_EMBED_AGGREGATION=centroid` `product-service` хранит **один** image-вектор на товар вместо N. При update/delete фото вектор пересчитывается:

```sql
embedding = normalize(mean(image_vec_i for i in 0..image_count))
```

Тогда `product_image_embeddings.PRIMARY KEY` — это `product_id`, без `image_idx`. SQL гибрида тот же, без `MAX` и `GROUP BY`.

Centroid дешевле по индексу (×N меньше строк) и вычислительно (один вызов provider-а на агрегацию не нужен — усреднение делаем сами), но размывает ракурсы. Стартуем с `per_image`, переключаемся на `centroid` если индекс по image-векторам станет узким местом по памяти/latency. Миграция между режимами — отдельный one-shot job, не runtime.

#### Internal endpoint-ы

| Endpoint | Кто вызывает | Что делает |
|---|---|---|
| `GET /products` | `stylist-service`, frontend | Обычный фильтрованный поиск по товарам |
| `GET /products/{id}` | `stylist-service`, frontend | Полная карточка товара |
| `GET /products/filter-options` | `stylist-service`, frontend | Реальные фасеты для брендов, категорий, размеров и т.д. |
| `POST /internal/products/semantic-search` | `stylist-service` | Принимает query vector + filters, считает hybrid score (text + image) и возвращает top-k |
| внутренние jobs `embed_product_text` / `embed_product_images` | сам `product-service` | Пересчитывают text- и image-вектора (см. ниже секцию «Поток индексации») |

`POST /internal/products/semantic-search`

Заголовки: `X-Internal-Token: <shared-secret>`, опционально `X-User-Id` для применения визибилити-правил.

Запрос:
```json
{
  "vector": [0.012, -0.043, "..."],
  "top_k": 10,
  "filters": {
    "category": "bags",
    "brand": null,
    "tags": ["leather"],
    "size": null,
    "condition": null,
    "price_min": null,
    "price_max": 10000
  }
}
```

Ответ:
```json
{
  "items": [
    {
      "id": "uuid",
      "score": 0.87,
      "score_breakdown": {
        "text": 0.81,
        "image": 0.93,
        "best_image_idx": 2
      },
      "name": "Vintage Coach bag",
      "brand": "Coach",
      "category": "bags",
      "price": 5500,
      "highlight_fields": ["leather", "90s", "crossbody"]
    }
  ]
}
```

Контракт фиксирует:
- `vector` — ровно `MULTIMODAL_EMBED_DIM`, иначе `400 invalid_vector_dim`.
- product-service сам отфильтровывает `is_deleted = true` и `status != 'ready'`.
- `score` — итоговая взвешенная hybrid-метрика, нормирована в `[0, 1]` (т.к. оба слагаемых cosine-similarity в `[0, 1]` после `1 - <=>`).
- `score_breakdown.text` / `score_breakdown.image` — компоненты до взвешивания. Используются `stylist-service` для аргументации в LLM («подошло по фото», «подошло по описанию») и в логах/метриках для тюнинга весов.
- `score_breakdown.best_image_idx` — индекс фото, которое дало MAX image-score (`null` в `centroid`-режиме). LLM может использовать его, чтобы попросить фронт показать именно этот ракурс.
- `highlight_fields` — короткий перечень полей, по которым товар «попал» в выборку (для аргументации LLM).

## Tool use

Четыре функции, которые LLM может звать:

| Tool | Аргументы | Что делает |
|---|---|---|
| `search_products` | `category?, brand?, tags?, price_min?, price_max?, size?, condition?` | Точечный SQL-фильтр через `product-service` |
| `semantic_search` | `query: str, top_k: int = 10, filters?` | Эмбеддит запрос и вызывает semantic endpoint `product-service` |
| `get_product_details` | `id: UUID` | Полная карточка из `product-service` |
| `list_facets` | `field: "tags" \| "brands" \| "categories"` | Реальные значения, чтобы LLM не галлюцинировал |

Все аргументы валидируются Pydantic. Если LLM прислал неизвестный тег/бренд — возвращаем tool error, модель переформулирует.

### Системный промпт

Системный промпт фиксированный, с инжекцией только `{user_role}` и (на старте новой сессии) свежих фасетов. Скелет:

```
Ты — стилист-консультант маркетплейса винтажной и second-hand одежды.
Отвечай по-русски. Цены в рублях. Не выдумывай товары: упоминай только те,
что реально вернулись из tools.

У тебя есть инструменты:
- search_products — точечные фильтры (бренд, категория, цена, размер). Используй,
  когда у пользователя конкретные критерии.
- semantic_search — поиск по смыслу запроса. Используй для размытых описаний
  ("в стиле 90-х", "что-то тёплое и уютное").
- get_product_details — полная карточка одного товара. Используй для drill-in
  ("расскажи подробнее про вторую").
- list_facets — список реальных значений (бренды, теги, категории).
  Зови, если не уверен, существует ли значение, до того как фильтровать.

Если пользователь сформулировал запрос слишком обобщённо — задай один уточняющий
вопрос вместо tool-use. Не задавай больше одного вопроса подряд.

При ответе с подбором товаров: 3–6 предложений, каждое — короткое объяснение,
чем товар хорош. Без markdown-таблиц. Карточки фронт отрисует сам по ids.
```

В контекст также подставляется `last_product_ids` от прошлого ответа (если есть) — короткой системной репликой `"Ранее показанные товары: [...]"`, чтобы drill-in работал без отдельного state machine.

## API

Все эндпоинты под `/api/stylist/...` — прикрыты nginx `auth_request`.

### `POST /api/stylist/chat` (SSE)

Запрос:

```json
{
  "conversation_id": "9f3a...",
  "message": "Хочу кожаную куртку в стиле 90-х до 8000"
}
```

Стрим событий:

```
event: token
data: {"text": "Подобрал "}

event: token
data: {"text": "несколько вариантов..."}

event: products
data: {"ids": ["uuid-1", "uuid-2", "uuid-3"]}

event: done
data: {}
```

Дополнительно:

```
event: ping
data: {}                                     # каждые 15 сек, чтобы держать соединение

event: error
data: {"code": "rate_limited", "message": "..."}    # затем сразу event: done
```

Возможные `code` в `error`:
- `rate_limited` — пользователь упёрся в дневной/минутный лимит.
- `upstream_unavailable` — OpenAI или product-service не отвечают.
- `bad_request` — неверный `conversation_id` или пустое сообщение.
- `internal` — всё остальное.

`event: done` отправляется **всегда** последним, в т.ч. после ошибки. Reconnect в MVP не поддерживается: если стрим оборван, фронт начинает новый запрос. Фронт рендерит текст по мере поступления токенов и параллельно подгружает карточки товаров отдельным запросом к `product-service` через nginx.

### `POST /api/stylist/conversations`

Создаёт новую сессию, возвращает `{ "conversation_id": "..." }`.

### `DELETE /api/stylist/conversations/{id}`

Чистит историю. Идемпотентно: `204` и для несуществующего id.

## Бюджет контекста и обрезка истории

Для каждого ответа собирается промпт по схеме:

```
[system prompt]            ~ 600 tokens
[summary of older turns]   ~ 0..200 tokens (опционально)
[last K user/assistant/tool messages]  ≤ HISTORY_TOKEN_BUDGET (6000)
[current user message]
```

Алгоритм:

1. Считаем токены через `tiktoken` для выбранной chat-модели.
2. Откидываем старые сообщения, пока сумма не влезет в `HISTORY_TOKEN_BUDGET`.
3. Если откинули ≥ 3 сообщения — синхронно зовём cheap-model (`gpt-4o-mini`, `max_tokens=200`) с просьбой суммаризовать выкинутый префикс. Сохраняем как одно «system: summary» сообщение в начале.
4. Большие tool-результаты (списки товаров) обрезаем до `top_k` (для `search_products` — до 20). Полную карточку из `get_product_details` оставляем как есть.
5. Сообщения, в которых уже есть только id-шник товара (после обрезки), помечаем `pruned=true`, чтобы не подавать LLM «огрызки» полных карточек.

Это разделение позволяет держать стабильный prompt-cost (~ $0.001 на типовой ответ при `gpt-4o-mini`) и не уезжать в context overflow на длинных диалогах.

## Обработка ошибок и сбои зависимостей

| Сбой | Поведение `stylist-service` |
|---|---|
| OpenAI 429 / `RateLimitError` | retry с экспоненциальным backoff (3 попытки, jitter), затем `event: error code=rate_limited` |
| OpenAI 5xx / network timeout | retry x2, затем `event: error code=upstream_unavailable` |
| Multimodal embed 429 / 5xx / timeout | retry x2 с backoff; затем `semantic_search` возвращает tool-error → LLM переключается на `search_products` |
| product-service 5xx | tool вызов возвращает `{"error": "...", "retriable": true}` → LLM решает: повторить, переформулировать, ответить без подбора |
| product-service вернул 0 товаров | tool возвращает `{"items": []}` — LLM объясняет «не нашёл» и предлагает уточнение |
| Невалидные tool-args | tool возвращает `{"error": "validation", "schema": {...}}` — модель повторяет с правкой |
| LLM зациклился на tool calls | hard-cap `MAX_TOOL_ITERATIONS`, далее принудительно завершаем с `tool_choice="none"` |
| Истёк лимит пользователя | сразу `event: error code=rate_limited`, без OpenAI-запроса |
| Истёк дневной cap query-эмбеддингов (`QUERY_EMBED_DAILY_CAP`) | `semantic_search` отвечает tool-error, LLM переключается на `search_products` |
| Невалидный `conversation_id` | `event: error code=bad_request` и `done` |

Все retries покрыты circuit-breaker-ом: если за 60 сек > 50% запросов к OpenAI/product-service падают, временно «пробиваем» — отдаём пользователю «сервис временно недоступен» без ожидания таймаутов.

## Диаграммы

### Поток 1. Точечный запрос (фильтры)

```mermaid
sequenceDiagram
    actor U as Покупатель
    participant FE as Frontend
    participant N as Nginx
    participant AI as stylist-service
    participant OAI as OpenAI
    participant PS as product-service

    U->>FE: "Кожаная сумка до 10к"
    FE->>N: POST /api/stylist/chat (SSE, JWT)
    N->>AI: forward
    AI->>AI: verify JWT, load history (in-memory)
    AI->>OAI: chat.completions.create(stream, tools=[...])
    OAI-->>AI: tool_call: search_products(category="bags", tags=["leather"], price_max=10000)
    AI->>PS: GET /products?category=bags&tags=leather&price_max=10000
    PS-->>AI: [products...]
    AI->>OAI: tool_result(products)
    OAI-->>AI: stream of tokens + product ids
    AI-->>FE: SSE: token... token... products... done
    FE->>PS: GET /products/{id} (для карточек, параллельно)
    FE->>U: чат + карточки
```

### Поток 2. Размытый запрос (semantic search)

Коротко:

```text
user request
  -> stylist-service
  -> LLM выбирает semantic_search
  -> stylist-service делает embedding запроса через multimodal-провайдера
     (один и тот же провайдер, что и в product-service embedder — иначе query
      окажется в чужом векторном пространстве)
  -> stylist-service отправляет query vector в product-service
  -> product-service считает hybrid score (w_text * text_cos + w_image * MAX(image_cos))
     по pgvector + применяет фильтры
  -> product-service возвращает подходящие товары + score_breakdown (text / image / best_image_idx)
  -> stylist-service стримит ответ + product ids на фронт
  -> frontend подгружает карточки товаров из product-service
```

Внутренний semantic endpoint лучше возвращает не только ids, а короткий payload для LLM:

```json
[
  {
    "id": "uuid",
    "score": 0.87,
    "name": "Vintage Coach bag",
    "brand": "Coach",
    "price": 5500,
    "reason_fields": ["leather", "90s", "crossbody"]
  }
]
```

`stylist-service` использует этот payload, чтобы объяснить подбор. На фронт через SSE достаточно отправить `product ids`, а карточки фронт загрузит обычным запросом к `product-service`.

```mermaid
sequenceDiagram
    participant AI as stylist-service
    participant OAI as OpenAI
    participant ME as Multimodal Embed API
    participant PS as product-service

    AI->>OAI: chat (user: "что-то в духе 90-х, тёплое")
    OAI-->>AI: tool_call: semantic_search(query="90s style warm vintage", top_k=10)
    AI->>ME: embed(text=query)
    ME-->>AI: vector(MULTIMODAL_EMBED_DIM)
    AI->>PS: POST /internal/products/semantic-search (vector, top_k, filters)
    PS->>PS: hybrid SQL: text_cos + MAX(image_cos), фильтры, ORDER BY score
    PS-->>AI: [products + score_breakdown...]
    AI->>OAI: tool_result(products)
    OAI-->>AI: stream of tokens
```

### Поток 3. Drill-in по контексту

```mermaid
sequenceDiagram
    participant U as Покупатель
    participant AI as stylist-service
    participant OAI as OpenAI
    participant PS as product-service

    Note over AI: В Conversation хранится last_product_ids<br/>после прошлого ответа
    U->>AI: "расскажи подробнее про вторую"
    AI->>OAI: chat (с историей и last_product_ids в системном контексте)
    OAI-->>AI: tool_call: get_product_details(id=last_product_ids[1])
    AI->>PS: GET /products/{id}
    PS-->>AI: full product
    AI->>OAI: tool_result
    OAI-->>AI: stream of tokens
```

### Поток 4. Индексация эмбеддингов в product-service

Два независимых пайплайна — текстовый и image. Оба используют **одну и ту же** multimodal-модель (вариант B), поэтому полученные векторы сравнимы между собой и с query-вектором из `ai-service`.

#### 4a. Text-эмбеддинги

```mermaid
sequenceDiagram
    participant W as product-text-embedder (job)
    participant DB as product DB
    participant ME as Multimodal Embed API

    loop по расписанию (или триггер из product write flow)
        W->>DB: SELECT id, generate_product_text(id) FROM ready products
        W->>W: compute text_hash, skip unchanged
        W->>ME: embed(text=[texts where hash changed])
        ME-->>W: vectors[]
        W->>DB: UPSERT product_text_embeddings (product_id, embedding, text_hash)
    end
```

#### 4b. Image-эмбеддинги

Цепляется к уже существующему job-queue `product-service` (`upload_product_images`, `delete_product_images` — см. `src/jobs/`). На MVP — два хука и один backfill-job:

1. **Hook в `upload_product_images` job.** После успешного `put_object` всех вариантов в S3 — enqueue нового job `embed_product_images { product_id, image_indices: [...] }`.
2. **Hook в `delete_product_images` (по индексу или по продукту).** После удаления из S3 — `DELETE FROM product_image_embeddings WHERE product_id=$1 AND image_idx = ANY($2)` (или по `product_id` для cascade).
3. **Backfill-job `reindex_product_images`** — single-shot, перебирает `ready` товары, у которых строк в `product_image_embeddings` меньше, чем `product.image_count`, и догоняет.

```mermaid
sequenceDiagram
    participant U as upload_product_images job
    participant E as embed_product_images job
    participant S3 as S3
    participant DB as product DB
    participant ME as Multimodal Embed API

    U->>S3: put_object thumb/medium/full.webp по индексам
    U->>DB: INSERT job embed_product_images(product_id, image_indices)
    Note over E: воркер берёт enqueued job
    E->>S3: GET medium.webp по каждому image_idx
    E->>E: compute image_hash, skip unchanged
    E->>ME: embed(image=[bytes_or_url where hash changed])
    ME-->>E: vectors[]
    E->>DB: UPSERT product_image_embeddings (product_id, image_idx, embedding, image_hash)
```

Решение «base64 vs presigned URL для отправки картинки в API провайдера» — за конкретным провайдером:
- Cohere / Voyage принимают base64.
- Vertex `multimodalembedding@001` принимает GCS URI или base64.
- Локальный CLIP/SigLIP — bytes напрямую.

В режиме `IMAGE_EMBED_AGGREGATION=centroid` `embed_product_images` после получения вектора каждого фото не пишет N строк, а пересчитывает один центроид всех векторов товара и UPSERT-ит единственную строку. Промежуточные image-векторы можно держать в памяти job-а или сохранять отдельно для отладки — на MVP в памяти.

## Примеры разговоров

### Пример 1 — точечный запрос

**User:** Покажи кожаные сумки до 10000

**Внутри:**
- `search_products(category="bags", tags=["leather"], price_max=10000)`
- product-service отдал 4 товара

**Assistant:**
> Нашёл 4 кожаных сумки в бюджете:
> 1. Винтажная Coach (5500₽) — небольшая через плечо, отличное состояние
> 2. Furla 90-х (8200₽) — структурная, кофейный цвет
> 3. ...
>
> [карточки 4 товаров]

### Пример 2 — размытый

**User:** Хочу что-нибудь в стиле 90-х, тёплое

**Внутри:**
- `semantic_search(query="90s style warm clothing vintage", top_k=10)`
- `stylist-service` получил query embedding через multimodal-провайдера
- `product-service` посчитал hybrid score: text-similarity по `generate_product_text` + MAX по фото товара, отфильтровал по правилам витрины и вернул топ-10 с `score_breakdown`
- LLM использует `score_breakdown` для аргументации («подошло по описанию», «подошло по виду»)

**Assistant:**
> Под «тёплое из 90-х» хорошо подойдут:
> 1. Шерстяной кардиган Benetton — крупная вязка, оверсайз
> 2. Пуховик Moncler 1996 — хаки, размер M
> 3. ...
>
> [карточки]

### Пример 3 — drill-in

**User:** Расскажи подробнее про вторую

**Внутри:**
- В сессии есть `last_product_ids` от прошлого ответа
- `get_product_details(id=last_product_ids[1])`

**Assistant:**
> Пуховик Moncler из коллекции 1996, цвет хаки, размер M. Подкладка целая, на правом рукаве лёгкая потёртость на манжете. Куплен в Милане в 2019 (`purchase_location`). Цена 14500₽.
>
> [карточка]

### Пример 4 — уточняющий вопрос

**User:** Хочу что-то на лето

**Внутри:**
- LLM понимает, что данных мало, не зовёт tools
- Сразу спрашивает уточнение

**Assistant:**
> Уточни: ищешь верх, низ или платье? И есть ли ориентир по бюджету?

## Защита и стоимость

- **JWT обязателен.** Без токена — 401. Терминируется на nginx (`auth_request → auth-service`), как у остальных сервисов.
- **Rate limit per user:** N сообщений/мин, M токенов/день. На старте — token bucket в памяти, дальше при необходимости Redis.
- **Cost guard:**
  - `max_tokens` на ответ
  - история обрезается до последних K сообщений + системный промпт
  - максимум T итераций tool-use в одном ответе (на случай зацикливания)
  - `QUERY_EMBED_DAILY_CAP` — дневной cap на query embeddings (multimodal-провайдер) в `stylist-service`
  - дневные cap-ы на text- и image-embedder в `product-service` (text- и image-цены у multimodal-провайдеров обычно отличаются — лимиты раздельные)
  - `image_hash` и `text_hash` — пересчёт только при изменении контента, неизменённые товары не тратят квоту
- **Валидация tool-args** через Pydantic — никаких сырых параметров от LLM в SQL/HTTP.
- **Internal endpoint-ы product-service** прикрыты `X-Internal-Token` (shared secret), нгинкс не проксирует `/internal/*` наружу.

## Наблюдаемость

Структурированные JSON-логи через `structlog`. Каждая запись несёт `request_id`, `conversation_id`, `user_id_hash` (sha256 первых 16 байт, чтобы не светить uuid в логах), `event_type`. Сырое содержимое сообщений — только на `LOG_LEVEL=debug`.

Метрики Prometheus на `/metrics`:

| Метрика | Тип | Лейблы |
|---|---|---|
| `stylist_chat_messages_total` | counter | `result` (`ok`, `rate_limited`, `error`) |
| `stylist_chat_duration_seconds` | histogram | — (от запроса до `done`) |
| `stylist_tool_calls_total` | counter | `tool`, `result` |
| `stylist_openai_tokens_total` | counter | `model`, `kind` (`prompt`/`completion`) |
| `stylist_openai_request_duration_seconds` | histogram | `model`, `kind` |
| `stylist_embed_requests_total` | counter | `provider`, `result` |
| `stylist_embed_request_duration_seconds` | histogram | `provider` |
| `stylist_product_service_requests_total` | counter | `endpoint`, `status` |
| `stylist_active_conversations` | gauge | — |
| `stylist_rate_limit_hits_total` | counter | `reason` |

Tracing: пробрасываем `X-Request-Id` (если nginx уже выставил) во все исходящие запросы и логи. Полноценный distributed tracing — вне MVP.

## Деплой

- `Dockerfile`: multi-stage, `python:3.12-slim` базой, установка зависимостей через `uv` или `pip --no-cache-dir`. `WORKDIR /app`, неприв. пользователь, `CMD ["uvicorn", "ai_service.main:app", "--host", "0.0.0.0", "--port", "8084"]`.
- `assets/docker-compose.ai.yml` — dev-вариант: build из контекста, env с дефолтами, depends_on `sc-product-service` (healthy) и `sc-auth-service` (healthy).
- `assets/docker-compose.ai.prod.yml` — prod-вариант: image из реестра, без `build`, restart `unless-stopped`.
- В корневом `docker-compose.yml` добавляется секция `sc-ai-service` (по образцу `sc-product-service`).
- В `nginx/nginx.conf` добавляется блок:

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

      # SSE
      proxy_buffering           off;
      proxy_cache               off;
      proxy_http_version        1.1;
      proxy_set_header Connection "";
      proxy_read_timeout        600s;
      proxy_send_timeout        600s;
  }
  ```

- Прод-секреты (`OPENAI_API_KEY`, `MULTIMODAL_EMBED_API_KEY`, `INTERNAL_API_TOKEN`) — только через `.env`/secret store, не коммитим.
- Healthcheck в compose: `curl -f http://localhost:8084/healthz`. `/healthz` не зовёт OpenAI и product-service, отвечает 200, если процесс жив.

## Тесты

- **Unit:** валидация tool-args (Pydantic), бюджет/обрезка истории, ассемблирование системного промпта, разбор tool-выходов, обработка краёв (пустой conversation_id, пустое сообщение).
- **Integration:** запуск ai-service против stub-а product-service (`pytest-httpserver` или мини-FastAPI), OpenAI замокан через VCR-кассеты для воспроизводимости. Отдельный «smoke» с реальным OpenAI запускается локально по флагу.
- **E2E (smoke):** скрипт `scripts/smoke-stylist.sh` поднимает `docker compose up`, прогоняет 4 примера из секции «Примеры разговоров» и проверяет, что каждый стрим завершается `event: done` без `event: error`. Не выполняется в обычном CI (нужен OpenAI ключ), запускается вручную перед релизом.
- **Контрактные тесты semantic-search endpoint** — pact-style: `stylist-service` и `product-service` гоняют общий JSON-фикстур и проверяют форму запроса/ответа. Достаточно одной таблицы `tests/contracts/semantic_search.json`, читаемой обеими сторонами.

## Локализация

- Каталог в БД на русском (товары, теги, категории, бренды).
- Системный промпт фиксирует: отвечать по-русски, цены в ₽.
- При семантическом поиске запрос эмбеддим как есть. Выбранный multimodal-провайдер должен поддерживать многоязычность (Cohere `embed-multilingual-v3.0` — да; Voyage и Vertex `multimodalembedding@001` — да; CLIP — англ. ориентированный, для русских запросов лучше M-CLIP / SigLIP-multilingual вариант). Это критерий при выборе модели в Phase 0b.
- Если когда-то понадобится английский интерфейс — это правка только промпта и фронта; продуктовые данные остаются локализованными в одной языковой версии.

## Что вне MVP

- Персистентная история чата (Redis / postgres).
- Горизонтальное масштабирование (нужен Redis для истории).
- Push-обновления product embeddings через шину событий вместо scheduled worker-а.
- Re-ranking результатов (cross-encoder поверх pgvector).
- Персонализация по истории просмотров и заказов.
- Метрики качества рекомендаций (CTR / add-to-cart по сессиям).
- Distributed tracing (OTel).
- Multi-tenant rate-limit (per role, per plan).
- **«Найди по фото»** — пользователь грузит картинку, чат подбирает похожие товары. Инфраструктура (multimodal joint space + image-вектора каждого фото) уже есть после Phase 0b, нужен только новый tool `image_search` и UI для аплоада. Откладываем как post-MVP, чтобы не разрастать MVP-scope.

## Открытые вопросы

- ~~Где терминировать JWT — на nginx или в `stylist-service`?~~ **Решено:** на nginx через `auth_request` к `auth-service`, как для product/user-service. `stylist-service` читает только `X-User-Id` и `X-User-Role`.
- ~~Раздельные пространства text/image (OpenAI text + любая image-модель) или single multimodal space?~~ **Решено:** single multimodal space (вариант B). Преимущества: cross-modal поиск (текст-запрос напрямую матчится на image-вектора товара), не нужны два разных эмбеддера в `ai-service`, общий `MULTIMODAL_EMBED_DIM` для индексных таблиц.
- **Какой multimodal-провайдер?** Кандидаты:
  - **Cohere Embed-3 multimodal** (`embed-multilingual-v3.0`) — multilingual из коробки, REST API, dim 1024. Стоимость средняя.
  - **Voyage `voyage-multimodal-3`** — REST API, dim 1024. Многоязычность ограничена, требует проверки на русском каталоге.
  - **Vertex `multimodalembedding@001`** — dim 1408, требует GCP-проекта и SA, картинки удобно слать GCS URI. Привязка к Google-стеку.
  - **Локальный open_clip / SigLIP (multilingual вариант)** — без внешних API, работает на CPU/GPU в отдельном контейнере. Бесплатно по запросам, но добавляет инфраструктуры. Текстовый encoder ограничен короткими токен-окнами — для длинного `generate_product_text` нужно проверять truncation.
  Решаем в Phase 0b после замера качества top-k на 30-запросном eval-датасете и оценки месячной стоимости при текущем размере каталога.
- **Per-image (MAX) vs centroid?** Стартуем с `per_image` — лучше ловит ракурсы. Если индекс по картинкам станет узким местом по памяти/latency — переключаемся на `centroid`. Решение фиксируется по метрикам после Phase 4.
- **`image_hash` от `medium.webp` или от raw?** Берём `medium.webp` — стабильнее (raw зависит от формата исходника, повторный re-upload того же фото не должен триггерить переиндексацию).
- **Веса гибрида `HYBRID_TEXT_WEIGHT` / `HYBRID_IMAGE_WEIGHT`** — стартуем с 0.5 / 0.5, тюним на 30-запросном eval-датасете в Phase 4.
- В текст для эмбеддинга — включаем `description` целиком или только ключевые поля? Гипотеза: ключевые поля + первые 500 символов description. Замерим качество top-k на ручном наборе из 30 запросов. (С multimodal моделью ограничение токенов на text-side может быть жёстче чем у OpenAI — это аргумент в пользу более компактного текста.)
- Нужен ли re-ranking шаг после semantic_search для MVP или достаточно top-k от pgvector? Проверяем на 30-запросном наборе после Phase 4.
- Как запускать product embeddings worker в MVP: внутри `product-service` по расписанию или отдельным internal job-контейнером? Склоняемся к internal job в составе `product-service` бинарника, активируется флагом `--mode=embedder`. Image-сторона лучше укладывается в существующий postgres-job-queue (`embed_product_images` job), text-сторона может быть scheduled worker.
- Защита internal endpoint-ов: достаточно ли `X-Internal-Token` или нужен mTLS? На MVP — токен + сетевая изоляция docker network.

## TODO: product embeddings

Текущее состояние в `product-service`:
- есть helper `generate_product_text(p_product_id)`;
- картинки лежат в S3 в трёх вариантах (`thumb`/`medium`/`full`.webp) по пути `products/{product_id}/{n}/`, число хранится в `product.image_count`, управляются job-queue (`upload_product_images`, `delete_product_images` в `src/jobs/`);
- при создании/изменении товара embeddings пока не считаются;
- таблиц `product_text_embeddings` / `product_image_embeddings` пока нет;
- `pgvector` extension пока не подключён;
- `similar_products` есть, но это таблица для уже рассчитанных похожих пар, не хранилище embeddings.

Что нужно добавить (вариант B, single multimodal space):

**Phase 0a (text):**
- подключить `pgvector` в product DB;
- добавить таблицу `product_text_embeddings(product_id, embedding vector(:dim), text_hash, updated_at)` под фиксированную `MULTIMODAL_EMBED_DIM`;
- добавить worker/job (`--mode=text-embedder`) в `product-service`, который берёт `generate_product_text(id)`, считает `text_hash`, вызывает выбранного multimodal-провайдера в text-mode и UPSERT-ит вектор;
- добавить internal endpoint `POST /internal/products/semantic-search` со скелетом hybrid SQL — на этом этапе `image_score = 0`, потому что image-таблица ещё пустая.

**Phase 0b (image):**
- добавить таблицу `product_image_embeddings(product_id, image_idx, embedding, image_hash, updated_at)` PK `(product_id, image_idx)`;
- добавить новый job-kind `embed_product_images` в `src/jobs/`;
- расширить `upload_product_images` — после успешной заливки в S3 enqueue `embed_product_images` в той же транзакции;
- расширить `delete_product_images` — удалять соответствующие строки из `product_image_embeddings`;
- добавить single-shot backfill `reindex_product_images` для существующих товаров;
- хук в hybrid SQL — добавить ветку `image_scores` с `MAX` агрегацией, дополнить ответ `score_breakdown`;
- env-флаги: `IMAGE_EMBED_AGGREGATION` (`per_image | centroid`), `HYBRID_TEXT_WEIGHT`, `HYBRID_IMAGE_WEIGHT`, `IMAGE_EMBED_DAILY_CAP`.

**Общее:**
- `MULTIMODAL_EMBED_PROVIDER` / `MULTIMODAL_EMBED_MODEL` / `MULTIMODAL_EMBED_DIM` / `MULTIMODAL_EMBED_API_KEY` — общие env, **должны совпадать** с `ai-service` (иначе query-вектор окажется в другом пространстве);
- решить, когда запускать text-переиндексацию: scheduled worker для MVP, позже событие из product write flow. Image-переиндексация уже триггерится через job-queue на upload/delete — отдельного scheduler-а не нужно.
