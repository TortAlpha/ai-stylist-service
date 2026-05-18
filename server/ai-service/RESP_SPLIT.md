# Stylist Service — разделение ответственности

Дополнение к `ROADMAP.md` и `DESIGN.md`. Делит работу на 3 роли:

- **FE** — Frontend (чат-интерфейс, SSE-клиент, карточки)
- **BE** — Backend / Tech (ai-service, изменения в product-service, nginx, compose, инфра)
- **QA** — Testing / Validation (тесты, eval-датасеты, нагрузка, smoke, мониторинг качества)

Цель документа — чтобы трое не наступали друг другу на ноги и было ясно, кто за какой артефакт отвечает.

---

## Scope каждой роли

### FE — Frontend

**Владеет:**
- Чат-страницей в клиенте (вход в сервис).
- SSE-клиентом: парсинг `event: token / products / ping / error / done`.
- Стейтом разговора на стороне фронта (`conversation_id`, render истории, индикаторы «печатает»).
- Карточками товаров: после `event: products` подгружает `GET /products/{id}` параллельно для каждого id, рендерит с лоадерами.
- Обработкой ошибок UX-side: rate_limited → баннер «лимит исчерпан», upstream_unavailable → «сервис временно недоступен», bad_request → «попробуйте перефразировать».
- Reconnect-стратегией (для MVP — кнопка «попробовать ещё раз», без авто-resume).
- Локализацией интерфейса (тексты по-русски, ₽, форматы дат/чисел).
- Доступностью (a11y) для чата.

**Не владеет:**
- SSE-протоколом и набором event-кодов (это контракт от BE — FE его потребляет, изменения через BE).
- Содержимым системного промпта.
- Структурой ответа `GET /products/{id}` (это контракт product-service — FE использует, не меняет).

### BE — Backend / Tech

**Владеет:**
- Всем кодом `server/ai-service/` (FastAPI app, tool-use loop, `multimodal_client`, конфиг, наблюдаемость).
- Изменениями в `server/product-service/` для Phase 0a/0b: миграции pgvector, `MultimodalEmbedder`-клиент, text-embedder job, новый `embed_product_images` job-kind + хуки в существующие `upload_product_images` / `delete_product_images`, internal endpoint `POST /internal/products/semantic-search` с hybrid SQL.
- **Реализацией клиента к multimodal-провайдеру** (Cohere `embed-multilingual-v3.0`, dim 1024 — выбор зафиксирован в DESIGN.md). Если post-MVP встанет вопрос о смене провайдера — BE готовит миграцию с полной переиндексацией.
- Системным промптом v0–v1 (содержимое — обсуждается с QA, но коммитит BE).
- `nginx/nginx.conf` — секция `/api/stylist/`, SSE-настройки.
- `assets/docker-compose.ai.yml`, `docker-compose.ai.prod.yml`, корневой compose.
- Управлением секретами: `OPENAI_API_KEY`, `MULTIMODAL_EMBED_API_KEY`, `INTERNAL_API_TOKEN` в dev/prod env.
- Метриками Prometheus и структурированными логами.
- Деплоем на стейдж/прод.

**Не владеет:**
- UX-решениями в чат-интерфейсе.
- Eval-датасетом для промпта (его собирает QA).
- Решением «работает или нет» — это критерии QA на каждой фазе.

### QA — Testing / Validation

**Владеет:**
- Pytest unit-тестами (валидация tool-args, бюджет истории, conversation TTL, парсинг SSE).
- Integration-тестами против stub product-service, замоканного OpenAI (VCR-кассеты) и stub `MultimodalEmbedder`.
- Контрактным тестом `tests/contracts/semantic_search.json` — общий для AI и PS, включает поле `score_breakdown` (поддерживает QA, читают обе стороны).
- Smoke-тестом `scripts/smoke-stylist.sh` (4 примера из DESIGN.md → `event: done` без error).
- Eval-датасетом для качества подбора: 30+ запросов с ручной разметкой (релевантно/нет), разбит на text-преобладающие (бренд / категория / цена), vision-преобладающие (цвет / принт / силуэт) и смешанные. Прогоны по semantic_search и search_products при разных `HYBRID_TEXT_WEIGHT` / `HYBRID_IMAGE_WEIGHT` для тюнинга в Phase 4.
- Eval-спайком в Phase 0a по подтверждению качества Cohere `embed-multilingual-v3.0` на русском каталоге: те же 30 запросов прогоняются на выбранном провайдере, измеряется recall@10 и стартовые `HYBRID_TEXT_WEIGHT` / `HYBRID_IMAGE_WEIGHT`. Если recall@10 окажется неудовлетворительным — поднимать вопрос смены провайдера (требует пересмотра DESIGN.md и downtime для переиндексации).
- Нагрузкой: локальный load-тест на 50 RPS (для Phase 5).
- Мониторингом качества в проде: дашборд по `stylist_*` метрикам, алерты на p95 latency и error rate.
- Регрессионным контролем: при смене chat-модели (gpt-4o-mini → gpt-4o) и при смене multimodal-провайдера прогон evaluation датасета и сравнение метрик. Смена multimodal-провайдера = полная переиндексация text + image (планируется как отдельный downtime).
- Финальной приёмкой каждой фазы — пункт «Готово, когда…» из ROADMAP.md.

**Не владеет:**
- Реализацией кода (тесты — да, прод-код — нет).
- Содержимым промпта (но даёт фидбек: «на запрос X промпт даёт Y, ожидалось Z»).

---

## Разбивка по фазам

| Фаза | FE | BE | QA |
|---|---|---|---|
| **P0a** pgvector + text embeddings | — | миграция `product_text_embeddings` (vector(1024)), `MultimodalEmbedder`-клиент под Cohere, text-embedder job, `/internal/semantic-search` v0 (только text) | eval-спайк на 30 запросов для подтверждения качества Cohere `embed-multilingual-v3.0` на русском каталоге; unit-тесты `text_hash` стабильность; контроль `is_deleted/!ready`; latency top-10 < 100ms p95 |
| **P0b** image embeddings | — | миграция `product_image_embeddings`, `embed_product_images` job, хуки в upload/delete, `reindex_product_images` backfill, hybrid SQL + `score_breakdown` | unit-тесты `image_hash`; e2e на upload/delete → строки появляются/пропадают; latency top-10 < 150ms p95 при `image_count=4`; контроль graceful degradation при пустой image-таблице |
| **P1** каркас ai-service | стаб-страница `/stylist` (заглушка), проверить, что 401 без JWT | `pyproject.toml`, Dockerfile, FastAPI скелет, `/healthz`, `/metrics`, nginx-блок, compose | health-check тест, e2e: `whoami` без/с JWT |
| **P2** chat без tools | SSE-клиент, рендер потока токенов, базовый UI чата, `POST/DELETE /conversations` | conversation store, OpenAI-обёртка, SSE handler, budget trim, системный промпт v0 | unit на budget/TTL, integration с моком OpenAI, ручной smoke «привет» |
| **P3** точечный поиск | `event: products` → подгрузка карточек через product-service, рендер списка | httpx-клиент, tools `search_products`/`list_facets`, tool-use loop, hard-cap iterations | integration: stub PS → пример 1; контроль, что невалидный tag не вешает loop |
| **P4** hybrid semantic + drill-in | follow-up-сообщения с подгрузкой контекста, drill-in UX (ссылка на «расскажи подробнее про N»); опционально — отображение `score_breakdown` (бэйджи «по описанию» / «по виду») | `multimodal_client.py`, `semantic_search`, `get_product_details`, `last_product_ids`, query embed cap; парсинг `score_breakdown` для аргументации в LLM | контрактный тест `semantic_search.json` со `score_breakdown`; eval-датасет 30 запросов в трёх категориях (text / vision / mixed); тюнинг весов; примеры 2 и 3 в smoke |
| **P5** лимиты и устойчивость | UX rate_limited / upstream_unavailable баннеры, отображение `event: error` | rate-limit, circuit-breaker (отдельно для OpenAI и multimodal embed), метрики, логи без user content на info | load-тест 50 RPS, отключение PS/OpenAI/multimodal-embed и проверка graceful degradation |
| **P6** релиз | прод-сборка фронта, проверка прод-конфига | прод-compose, `.env.example` (включая `MULTIMODAL_*`), README, secrets в проде | прохождение smoke на проде, дашборд метрик зелёный |
| **P7+** post-MVP | интеграция с историей просмотров, feedback UI (лайк/дизлайк), UI для «найди по фото» (image upload) | Redis для истории, push-обновление text-эмбеддингов, image-search tool, re-ranking | расширение eval-датасета (включая визуальные image-queries), A/B-инфра, метрики качества рекомендаций |

---

## RACI по ключевым артефактам

`R` — делает, `A` — accountable (финальная ответственность), `C` — consulted, `I` — informed.

| Артефакт | FE | BE | QA |
|---|---|---|---|
| Системный промпт | C | R/A | C |
| SSE wire-протокол (events, codes) | C | R/A | C |
| Tool JSON schemas | I | R/A | C |
| Чат-страница и UI | R/A | C | C |
| `event: products` → карточки на фронте | R/A | C | I |
| Миграция pgvector + `product_text_embeddings` | I | R/A | C |
| Миграция `product_image_embeddings` + хуки в upload/delete | I | R/A | C |
| Выбор multimodal-провайдера (зафиксирован: Cohere `embed-multilingual-v3.0`) | I | R/A | C |
| `MultimodalEmbedder`-клиент (PS и AI) | I | R/A | C |
| Hybrid SQL и `score_breakdown` | I | R/A | C |
| `/internal/semantic-search` контракт | I | R/A | C |
| Контрактный тест `semantic_search.json` (со `score_breakdown`) | I | C | R/A |
| Eval-датасет (30+ запросов: text / vision / mixed) | I | C | R/A |
| Тюнинг `HYBRID_*_WEIGHT` | I | C | R/A |
| Load-тест | I | C | R/A |
| Smoke-скрипт `scripts/smoke-stylist.sh` | I | C | R/A |
| nginx `/api/stylist/` location | I | R/A | C |
| docker-compose dev/prod | I | R/A | I |
| `.env.example` и секреты (OpenAI + Multimodal Embed) | I | R/A | I |
| README в `ai-service` | I | R/A | C |
| Прод-деплой | I | R/A | C |
| Дашборд метрик и алерты | I | C | R/A |
| Промпт-регрессии при смене chat-модели | I | C | R/A |
| Полная переиндексация при смене multimodal-провайдера | I | R | A |

---

## Точки стыковки (где трое договариваются)

Это не зоны ответственности, а контракты — должны быть согласованы заранее, чтобы не переделывать.

```mermaid
flowchart LR
    subgraph FE_BE["FE ↔ BE"]
        S1["SSE wire-протокол:<br/>event types, поля,<br/>error codes"]
        S2["Форма ответа /products/{id}<br/>(уже есть в product-service)"]
    end

    subgraph BE_QA["BE ↔ QA"]
        S3["Контракт semantic_search<br/>+ score_breakdown<br/>(JSON-фикстура)"]
        S4["Метрики и их лейблы<br/>(чтобы алерты привязать)"]
        S5["Тестовая среда<br/>(stub PS, VCR-кассеты,<br/>stub MultimodalEmbedder)"]
        S8["Eval-качества Cohere<br/>на русском каталоге<br/>(спайк в P0a)"]
    end

    subgraph FE_QA["FE ↔ QA"]
        S6["UX states: loading,<br/>rate_limited, error"]
        S7["e2e-сценарии<br/>(4 примера из DESIGN.md)"]
    end

    classDef joint fill:#fef9c3,stroke:#ca8a04,color:#000
    class S1,S2,S3,S4,S5,S6,S7,S8 joint
```

Каждая стыковка — отдельный мини-контракт, фиксируется PR-ом или строчкой в `DESIGN.md` ещё до того, как кто-то начинает писать соответствующий код.

---

## Definition of Done на фазу — кто что подтверждает

```mermaid
flowchart TB
    A[Phase N в работе]
    A --> BE_DONE{BE: артефакты<br/>замержены?}
    BE_DONE -- нет --> A
    BE_DONE -- да --> FE_DONE{FE: UX-сценарии<br/>работают вживую?}
    FE_DONE -- нет --> A
    FE_DONE -- да --> QA_DONE{QA: «Готово, когда»<br/>выполнены, тесты зелёные?}
    QA_DONE -- нет --> A
    QA_DONE -- да --> MERGE[Merge в master,<br/>переход на Phase N+1]

    classDef be fill:#dcfce7,stroke:#16a34a,color:#000
    classDef fe fill:#e8f4ff,stroke:#3b82f6,color:#000
    classDef qa fill:#fef9c3,stroke:#ca8a04,color:#000
    classDef done fill:#fff,stroke:#374151,stroke-width:2px,color:#000
    class BE_DONE be
    class FE_DONE fe
    class QA_DONE qa
    class MERGE done
```

QA — последняя дверь. Без её «да» фаза не закрывается, даже если код смержен.

---

## Что НЕ должно случаться (анти-паттерны)

- **FE пишет SSE-протокол под себя** и просит BE подстроиться. Протокол — собственность BE, FE может предложить изменение через PR в DESIGN.md.
- **BE пишет тесты «для галочки»** и считает, что фаза готова. Приёмка — у QA, по критериям ROADMAP.md.
- **QA правит прод-код**, чтобы тест прошёл. Если тест красный, QA открывает issue / PR с воспроизведением, BE чинит.
- **Любая роль делает работу другой роли в обход**: «я быстренько подкручу». Фиксируем в issue, обсуждаем, потом делаем.
- **Промпт меняется без прогона eval-датасета** на нём. После Phase 4 любая правка системного промпта = QA-прогон.

---

## Размер ролей

Не равный, и это нормально.

- BE — самый большой объём (≈ 60% работы по объёму кода). Сосредоточен на P0a + P0b + P2–P5.
- FE — ≈ 25%. Самая нагруженная фаза — P2 (новый UI чата) и P3 (карточки + `event: products`).
- QA — ≈ 15% по коду, но критическая роль на P4 (eval) и P5 (нагрузка) и постоянная роль в каждой приёмке.

В моменты пиков (например, релиз P6) роли могут заходить друг к другу — но только по согласованию, а не явочным порядком.
