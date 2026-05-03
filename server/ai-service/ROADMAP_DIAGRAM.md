# Stylist Service — roadmap в диаграммах

Визуальное дополнение к `ROADMAP.md`. Текстовые критерии готовности и список артефактов — там; здесь — связи, последовательность и состояние системы по фазам.

## Граф зависимостей фаз

```mermaid
flowchart LR
    P0["Phase 0<br/>pgvector в product-service<br/>(миграция, embedder, semantic-search)"]
    P1["Phase 1<br/>каркас ai-service<br/>(/healthz, nginx, compose)"]
    P2["Phase 2<br/>chat без tools<br/>(SSE, история, TTL)"]
    P3["Phase 3<br/>точечный поиск<br/>(search_products, list_facets)"]
    P4["Phase 4<br/>semantic + drill-in<br/>(semantic_search, get_product_details)"]
    P5["Phase 5<br/>лимиты и устойчивость<br/>(rate-limits, circuit-breaker)"]
    P6["Phase 6<br/>релиз"]
    PN["Phase 7+<br/>post-MVP<br/>(Redis, re-ranking, метрики качества)"]

    P0 --> P4
    P1 --> P2 --> P3 --> P4
    P4 --> P5 --> P6
    P6 --> PN

    classDef parallel fill:#e8f4ff,stroke:#3b82f6,stroke-width:2px,color:#000
    classDef serial fill:#fff,stroke:#374151,color:#000
    classDef post fill:#f9fafb,stroke:#9ca3af,stroke-dasharray:4 3,color:#000
    class P0,P1 parallel
    class P2,P3,P4,P5,P6 serial
    class PN post
```

Голубые узлы (P0, P1) могут идти параллельно. Серые (P7+) — после релиза, без жёсткого порядка.

## Критический путь

```mermaid
flowchart TB
    subgraph CP["Критический путь до релиза"]
        direction LR
        C1[P1] --> C2[P2] --> C3[P3] --> C4[P4] --> C5[P5] --> C6[P6]
    end

    subgraph PARA["Параллельная ветка"]
        D0[P0]
    end

    D0 -.merge.-> C4

    classDef cp fill:#fee2e2,stroke:#dc2626,stroke-width:2px,color:#000
    classDef par fill:#e8f4ff,stroke:#3b82f6,color:#000
    class C1,C2,C3,C4,C5,C6 cp
    class D0 par
```

Если Phase 0 не успевает к старту Phase 4 — релиз отодвигается. Это единственная межсервисная зависимость, всё остальное локально в `ai-service`.

## Условный таймлайн (Gantt)

Длительности — в условных «единицах работы» (1 ед. ≈ 1–2 рабочих дня), не в календарных днях. Для оценки порядка, не дат.

```mermaid
gantt
    title Roadmap Stylist Service
    dateFormat  X
    axisFormat  %s
    section product-service
    Phase 0 — pgvector + embedder        :p0, 0, 5
    section ai-service
    Phase 1 — каркас                      :p1, 0, 2
    Phase 2 — chat без tools              :p2, after p1, 3
    Phase 3 — search_products + facets    :p3, after p2, 3
    Phase 4 — semantic + drill-in         :p4, after p3, 3
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
    OAI[OpenAI]
    PS[product-service]
    EMB["product-embedder<br/>(scheduled)"]
    DB[(product DB<br/>+ pgvector)]

    FE -- SSE --> NG --> AI
    NG -- auth_request --> AS
    AI -- chat + embed --> OAI
    AI -- GET /products,<br/>POST /internal/semantic-search --> PS
    PS --> DB
    EMB -- embeddings.create --> OAI
    EMB -- UPSERT product_embeddings --> DB

    classDef p0 fill:#fef9c3,stroke:#ca8a04,stroke-width:2px,color:#000
    classDef p4 fill:#dcfce7,stroke:#16a34a,stroke-width:2px,color:#000
    class EMB,DB p0
    class AI p4
```

Жёлтым — компоненты Phase 0 (`product-embedder`, `pgvector` в product DB). Зелёным — то, что добавилось/расширилось в Phase 4.

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

    Phase3 --> Phase4: P4 добавляет semantic + drill-in
    state Phase4 {
        [*] --> chat4
        chat4 --> tool_call4: search / semantic / get_details / facets
        tool_call4 --> embed: если semantic_search
        embed --> ps_semantic
        ps_semantic --> chat4
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
        PS_P0["P0: pgvector,<br/>product_embeddings,<br/>embedder job,<br/>/internal/semantic-search"]
    end

    subgraph AI["ai-service (Python)"]
        AI_P1["P1: каркас, /healthz, /metrics,<br/>auth-headers"]
        AI_P2["P2: SSE chat, in-memory история,<br/>budget trim"]
        AI_P3["P3: search_products, list_facets,<br/>tool-use loop, products event"]
        AI_P4["P4: semantic_search, get_product_details,<br/>last_product_ids, embeddings cap"]
        AI_P5["P5: rate-limits, circuit breakers,<br/>SSE error events, метрики, логи"]
        AI_P1 --> AI_P2 --> AI_P3 --> AI_P4 --> AI_P5
    end

    subgraph NG["nginx"]
        NG_P1["P1: location /api/stylist/<br/>auth_request, SSE-tuning"]
    end

    subgraph CO["compose"]
        CO_P0["P0: sc-product-embedder"]
        CO_P1["P1: sc-ai-service в dev"]
        CO_P6["P6: sc-ai-service в prod,<br/>.env.example"]
    end

    PS_P0 --> AI_P4
    NG_P1 --> AI_P2
    CO_P1 --> AI_P2
    CO_P0 --> PS_P0

    classDef ps fill:#fef9c3,stroke:#ca8a04,color:#000
    classDef ai fill:#dcfce7,stroke:#16a34a,color:#000
    classDef ng fill:#e8f4ff,stroke:#3b82f6,color:#000
    classDef co fill:#fce7f3,stroke:#db2777,color:#000
    class PS_P0 ps
    class AI_P1,AI_P2,AI_P3,AI_P4,AI_P5 ai
    class NG_P1 ng
    class CO_P0,CO_P1,CO_P6 co
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
    R1["Phase 0 затягивается<br/>→ блокирует Phase 4"]
    R2["OpenAI ratelimits<br/>в проде → P5 надо раньше"]
    R3["Качество semantic_search<br/>низкое → нужен re-ranking<br/>(post-MVP вытягивается)"]
    R4["In-memory история теряется<br/>при рестарте → пользователи<br/>жалуются → Redis раньше"]

    R1 --> P4
    R2 --> P5
    R3 --> P7[Phase 7+]
    R4 --> P7

    classDef risk fill:#fee2e2,stroke:#dc2626,color:#000
    classDef phase fill:#e8f4ff,stroke:#3b82f6,color:#000
    class R1,R2,R3,R4 risk
    class P4,P5,P7 phase
```

Митигации описаны в `DESIGN.md` (секции «Обработка ошибок и сбои зависимостей» и «Что вне MVP»).
