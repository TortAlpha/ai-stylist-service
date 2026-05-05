# AI Service Lab

Учебный проект для разработки `ai-service` в экосистеме AvaVintage. Цель
сервиса - создать AI-стилиста для покупателей: он принимает запросы на
естественном языке, помогает подбирать товары, уточняет предпочтения и
возвращает рекомендации с карточками товаров.

Подробный дизайн описан в `server/ai-service/DESIGN.md`.

## Что делает сервис

`ai-service` работает как отдельный FastAPI-сервис, который:

- принимает чат-запросы от frontend через SSE;
- использует OpenAI API для генерации ответа и tool use;
- использует multimodal embedding-модель (общую с `product-service`) для query-эмбеддингов — текст и картинки в одном векторном пространстве;
- ходит в `product-service` за товарами, фильтрами, фасетами и hybrid semantic search (text + image);
- проверяет пользователя через JWT, выпущенный `auth-service`;
- хранит историю диалога in-memory для MVP;
- возвращает текстовый ответ и ids рекомендованных товаров.

Основной endpoint MVP:

```text
POST /api/stylist/chat
```

Ответ стримится событиями:

```text
token -> token -> products -> done
```

## Архитектура

```text
frontend
  -> nginx
  -> ai-service
      -> OpenAI API           (chat completion)
      -> Multimodal Embed API (query embedding — общий провайдер с product-service)
      -> product-service       (search, filters, hybrid semantic search)
      -> auth-service
```

`ai-service` не хранит копию каталога и не ходит напрямую в product DB.
Источником правды по товарам остаётся `product-service`.

## Основной стек

- FastAPI + Uvicorn
- OpenAI Python SDK
- Pydantic
- SSE через `sse-starlette`
- HTTP-клиент к `product-service`
- JWT авторизация
- in-memory conversations для MVP

## Product Embeddings

Semantic search проектируется так, чтобы embeddings товаров принадлежали
`product-service`. В product DB добавляются две таблицы с `pgvector`:

- `product_text_embeddings` — один вектор на товар, считается по
  `generate_product_text(id)`;
- `product_image_embeddings` — по строке на каждое фото товара (по
  `medium.webp` варианта), агрегация при поиске — `MAX` cosine-similarity.

Обе таблицы и query-эмбеддинг в `ai-service` используют **одну и ту же**
multimodal embedding-модель (вариант B — единое векторное пространство для
текста и картинок). Это позволяет text-запросу из чата напрямую матчиться на
визуальные признаки товара. Конкретный провайдер (Cohere Embed-3 / Voyage
Multimodal / Vertex `multimodalembedding@001` / open_clip+SigLIP)
фиксируется в Phase 0a после спайка по качеству и стоимости.

`ai-service` эмбеддит только пользовательский запрос и отправляет query
vector во внутренний endpoint `product-service`, который считает hybrid
score `w_text * text_cos + w_image * MAX(image_cos)` и возвращает товары
вместе со `score_breakdown`.

## Tool Use

LLM сможет вызывать ограниченный набор инструментов:

- `search_products` - точный поиск по фильтрам;
- `semantic_search` - поиск по смыслу через embeddings;
- `get_product_details` - получение полной карточки товара;
- `list_facets` - получение реальных брендов, категорий и тегов.

Все аргументы tools должны валидироваться через Pydantic.

## MVP Ограничения

- история диалогов хранится только в памяти процесса;
- после рестарта сервиса история пропадает;
- горизонтальное масштабирование потребует Redis или другое внешнее хранилище;
- rate limits и cost guards на старте можно реализовать in-memory;
- text- и image-embedder в `product-service`, миграции pgvector и
  hybrid semantic endpoint ещё нужно добавить (Phase 0a + 0b);
- «найти по фото» (image upload в чат) — post-MVP, инфраструктура для этого
  закладывается уже в MVP, но tool/UI добавляется позже.
