# Stylist Service — roadmap в диаграммах

Визуальное дополнение к `ROADMAP.md`. Текстовые критерии готовности и список артефактов — там; здесь — связи, последовательность и состояние системы по фазам.

## Граф зависимостей фаз

```mermaid
flowchart LR
    P0A["Phase 0a<br/>pgvector + text embeddings<br/>(Cohere клиент,<br/>product_text_embeddings,<br/>text-embedder, semantic-search v0)"]
    P0B["Phase 0b<br/>image embeddings<br/>(product_image_embeddings,<br/>embed_product_images job,<br/>hybrid SQL)"]
    P1["Phase 1<br/>каркас ai-service<br/>(/healthz, nginx, compose)"]
    P2["Phase 2<br/>chat без tools<br/>(SSE, история, TTL)"]
    P3["Phase 3<br/>точечный поиск<br/>(search_products, list_facets)"]
    P4["Phase 4<br/>semantic + drill-in<br/>(hybrid semantic_search,<br/>get_product_details)"]
    P5["Phase 5<br/>лимиты и устойчивость<br/>(rate-limits, circuit-breaker)"]
    P6["Phase 6<br/>релиз"]
    PN["Phase 7+<br/>post-MVP<br/>(Redis, image-search tool, re-ranking)"]

    P0A --> P0B --> P4
    P1 --> P2 --> P3 --> P4
    P4 --> P5 --> P6
    P6 --> PN

    classDef parallel fill:#e8f4ff,stroke:#3b82f6,stroke-width:2px,color:#000
    classDef serial fill:#fff,stroke:#374151,color:#000
    classDef post fill:#f9fafb,stroke:#9ca3af,stroke-dasharray:4 3,color:#000
    class P0A,P1 parallel
    class P0B,P2,P3,P4,P5,P6 serial
    class PN post
```

Голубые узлы (P0a, P1) могут идти параллельно. P0b ждёт 0a (нужен `MultimodalEmbedder`-клиент и зафиксированный провайдер), но может идти параллельно с P2–P3. Серые (P7+) — после релиза, без жёсткого порядка.

## Критический путь

```mermaid
flowchart TB
    subgraph CP["Критический путь до релиза"]
        direction LR
        C1[P1] --> C2[P2] --> C3[P3] --> C4[P4] --> C5[P5] --> C6[P6]
    end

    subgraph PARA["Параллельная ветка (product-service)"]
        direction LR
        D0A[P0a] --> D0B[P0b]
    end

    D0B -.merge.-> C4

    classDef cp fill:#fee2e2,stroke:#dc2626,stroke-width:2px,color:#000
    classDef par fill:#e8f4ff,stroke:#3b82f6,color:#000
    class C1,C2,C3,C4,C5,C6 cp
    class D0A,D0B par
```

Если Phase 0a+0b не успевают к старту Phase 4 — релиз отодвигается. Это единственная межсервисная зависимость, всё остальное локально в `ai-service`. Внутри ветки P0b следует строго после P0a (общий `MultimodalEmbedder`-клиент и зафиксированный провайдер).

## Условный таймлайн (Gantt)

Длительности — в условных «единицах работы» (1 ед. ≈ 1–2 рабочих дня), не в календарных днях. Для оценки порядка, не дат.

```mermaid
gantt
    title Roadmap Stylist Service
    dateFormat  X
    axisFormat  %s
    section product-service
    Phase 0a — pgvector + text embed     :p0a, 0, 4
    Phase 0b — image embed + hybrid SQL  :p0b, after p0a, 3
    section ai-service
    Phase 1 — каркас                      :p1, 0, 2
    Phase 2 — chat без tools              :p2, after p1, 3
    Phase 3 — search_products + facets    :p3, after p2, 3
    Phase 4 — hybrid semantic + drill-in  :p4, after p3, 3
    Phase 5 — лимиты и устойчивость       :p5, after p4, 3
    Phase 6 — релиз                       :p6, after p5, 1
    section post-MVP
    Phase 7+                              :p7, after p6, 5
```

## Эволюция архитектуры по фазам

### После Phase 1

```mermaid
flowchart LR
    FE[Frontend]
    NG[nginx]
    AS[auth-service]
    PS[product-service]
    AI["ai-service<br/>(/healthz, /metrics,<br/>/api/stylist/whoami)"]

    FE --> NG
    NG -- auth_request --> AS
    NG --> PS
    NG --> AI

    classDef new fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#000
    class AI new
```

Сервис поднимается, проходит auth, но ничего полезного не делает.

### После Phase 3

```mermaid
flowchart LR
    FE[Frontend]
    NG[nginx]
    AS[auth-service]
    PS[product-service]
    AI["ai-service<br/>SSE chat<br/>+ search_products<br/>+ list_facets"]
    OAI[OpenAI]

    FE -- SSE --> NG
    NG -- auth_request --> AS
    NG --> AI
    AI -- chat stream + tools --> OAI
    AI -- GET /products,<br/>GET /products/filter-options --> PS
    FE -- GET /products/{id} --> NG --> PS

    classDef new fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#000
    class AI new
```

Точечные запросы работают end-to-end. Размытых запросов и drill-in ещё нет.

### После Phase 4 (MVP-функциональность)

```mermaid
flowchart LR
    FE[Frontend]
    NG[nginx]
    AS[auth-service]
    AI["ai-service<br/>full tool-use loop"]
    OAI[OpenAI<br/>chat only]
    ME[Multimodal Embed API]
    PS[product-service]
    TEMB["product-text-embedder<br/>(scheduled)"]
    IEMB["embed_product_images<br/>job (триггер: upload/delete)"]
    S3[(S3<br/>products/{id}/{n}/{variant}.webp)]
    DB[(product DB + pgvector<br/>product_text_embeddings,<br/>product_image_embeddings)]

    FE -- SSE --> NG --> AI
    NG -- auth_request --> AS
    AI -- chat --> OAI
    AI -- embed_text(query) --> ME
    AI -- GET /products,<br/>POST /internal/semantic-search --> PS
    PS -- hybrid SQL: text + MAX(image) --> DB
    TEMB -- embed_text(generate_product_text) --> ME
    TEMB -- UPSERT product_text_embeddings --> DB
    IEMB -- GET medium.webp --> S3
    IEMB -- embed_image(bytes) --> ME
    IEMB -- UPSERT product_image_embeddings --> DB

    classDef p0a fill:#fef9c3,stroke:#ca8a04,stroke-width:2px,color:#000
    classDef p0b fill:#fde68a,stroke:#b45309,stroke-width:2px,color:#000
    classDef p4 fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#000
    class TEMB p0a
    class IEMB,S3 p0b
    class DB p0a
    class AI,ME p4
```

Жёлтым — компоненты Phase 0a (`product-text-embedder`, text-таблица в pgvector). Тёмно-жёлтым — Phase 0b (`embed_product_images` job, image-таблица, S3 как источник для картинок). Зелёным — то, что добавилось/расширилось в Phase 4 (multimodal-клиент в `ai-service`, hybrid score). OpenAI остался только в chat-канале.

## Покрытие сценариев из DESIGN.md

```mermaid
flowchart LR
    EX1["Пример 1<br/>«кожаные сумки до 10к»<br/>(точечный)"]
    EX2["Пример 2<br/>«в стиле 90-х, тёплое»<br/>(размытый)"]
    EX3["Пример 3<br/>«расскажи про вторую»<br/>(drill-in)"]
    EX4["Пример 4<br/>«хочу что-то на лето»<br/>(уточняющий вопрос)"]

    P2 --> EX4
    P3 --> EX1
    P3 --> EX4
    P4 --> EX2
    P4 --> EX3

    classDef ex fill:#fff,stroke:#374151,color:#000
    classDef ph fill:#e8f4ff,stroke:#3b82f6,color:#000
    class EX1,EX2,EX3,EX4 ex
    class P2,P3,P4 ph
```

Пример 4 (уточняющий вопрос) формально работает уже после Phase 2 — модель сама задаёт вопрос без tool-use. Полное качество — после обновления промпта в Phase 3.

## Tool-use loop (как становится сложнее по фазам)

```mermaid
stateDiagram-v2
    [*] --> Phase2: P2
    state Phase2 {
        [*] --> chat
        chat --> stream_tokens
        stream_tokens --> done
    }

    Phase2 --> Phase3: P3 добавляет tools
    state Phase3 {
        [*] --> chat3
        chat3 --> tool_call: search_products / list_facets
        tool_call --> chat3: tool_result
        chat3 --> stream_tokens3
        stream_tokens3 --> products_event
        products_event --> done3
    }

    Phase3 --> Phase4: P4 добавляет hybrid semantic + drill-in
    state Phase4 {
        [*] --> chat4
        chat4 --> tool_call4: search / semantic / get_details / facets
        tool_call4 --> mm_embed: если semantic_search
        mm_embed --> ps_hybrid: query vector
        ps_hybrid --> chat4: items + score_breakdown(text,image)
        tool_call4 --> chat4: иначе tool_result
        chat4 --> stream_tokens4
        stream_tokens4 --> products_event4
        products_event4 --> done4
    }
```

## Что включается и где (по компонентам)

```mermaid
flowchart LR
    subgraph PS["product-service (Rust)"]
        PS_P0A["P0a: pgvector,<br/>product_text_embeddings,<br/>MultimodalEmbedder client,<br/>text-embedder job,<br/>/internal/semantic-search v0"]
        PS_P0B["P0b: product_image_embeddings,<br/>embed_product_images job,<br/>хуки в upload/delete,<br/>hybrid SQL + score_breakdown"]
        PS_P0A --> PS_P0B
    end

    subgraph AI["ai-service (Python)"]
        AI_P1["P1: каркас, /healthz, /metrics,<br/>auth-headers"]
        AI_P2["P2: SSE chat, in-memory история,<br/>budget trim"]
        AI_P3["P3: search_products, list_facets,<br/>tool-use loop, products event"]
        AI_P4["P4: semantic_search (hybrid),<br/>multimodal_client, get_product_details,<br/>last_product_ids, query embed cap"]
        AI_P5["P5: rate-limits, circuit breakers,<br/>SSE error events, метрики, логи"]
        AI_P1 --> AI_P2 --> AI_P3 --> AI_P4 --> AI_P5
    end

    subgraph NG["nginx"]
        NG_P1["P1: location /api/stylist/<br/>auth_request, SSE-tuning"]
    end

    subgraph CO["compose"]
        CO_P0A["P0a: sc-product-text-embedder"]
        CO_P0B["P0b: hooks в существующий<br/>job-worker product-service"]
        CO_P1["P1: sc-ai-service в dev"]
        CO_P6["P6: sc-ai-service в prod,<br/>.env.example (MULTIMODAL_*)"]
    end

    PS_P0B --> AI_P4
    NG_P1 --> AI_P2
    CO_P1 --> AI_P2
    CO_P0A --> PS_P0A
    CO_P0B --> PS_P0B

    classDef ps fill:#fef9c3,stroke:#ca8a04,color:#000
    classDef ps2 fill:#fde68a,stroke:#b45309,color:#000
    classDef ai fill:#dcfce7,stroke:#16a34a,color:#000
    classDef ng fill:#e8f4ff,stroke:#3b82f6,color:#000
    classDef co fill:#fce7f3,stroke:#db2777,color:#000
    class PS_P0A ps
    class PS_P0B ps2
    class AI_P1,AI_P2,AI_P3,AI_P4,AI_P5 ai
    class NG_P1 ng
    class CO_P0A,CO_P0B,CO_P1,CO_P6 co
```

## Definition of Done — флоу принятия фазы

```mermaid
flowchart TB
    A[Phase N в работе] --> B{Все артефакты<br/>из ROADMAP.md<br/>созданы?}
    B -- нет --> A
    B -- да --> C{Все критерии<br/>«Готово, когда»<br/>выполнены?}
    C -- нет --> A
    C -- да --> D{Тесты зелёные<br/>+ smoke прошёл?}
    D -- нет --> A
    D -- да --> E{Метрики<br/>и логи живые?}
    E -- нет --> A
    E -- да --> F[Merge в master]
    F --> G[Phase N+1]

    classDef done fill:#dcfce7,stroke:#16a34a,color:#000
    classDef wip fill:#fef9c3,stroke:#ca8a04,color:#000
    class F,G done
    class A wip
```

## Риски на критическом пути

```mermaid
flowchart LR
    R1["Phase 0a/0b затягивается<br/>→ блокирует Phase 4"]
    R2["OpenAI ratelimits<br/>в проде → P5 надо раньше"]
    R3["Качество hybrid semantic_search<br/>низкое → re-ranking или<br/>тюнинг весов (post-MVP)"]
    R4["In-memory история теряется<br/>при рестарте → пользователи<br/>жалуются → Redis раньше"]
    R5["Стоимость multimodal embed<br/>(image-сторона) выше плана<br/>→ переход на centroid или<br/>самохостный CLIP"]
    R6["Provider lock-in: смена модели<br/>→ полная переиндексация<br/>(text + image), отдельный downtime"]

    R1 --> P4
    R2 --> P5
    R3 --> P7[Phase 7+]
    R4 --> P7
    R5 --> P5
    R6 --> P0[Phase 0a/0b]

    classDef risk fill:#fee2e2,stroke:#dc2626,color:#000
    classDef phase fill:#e8f4ff,stroke:#3b82f6,color:#000
    class R1,R2,R3,R4,R5,R6 risk
    class P0,P4,P5,P7 phase
```

Митигации описаны в `DESIGN.md` (секции «Обработка ошибок и сбои зависимостей» и «Что вне MVP»).
