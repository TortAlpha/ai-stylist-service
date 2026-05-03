# AI Service Lab

Учебный проект для разработки `ai-service` в экосистеме AvaVintage. Цель
сервиса - создать AI-стилиста для покупателей: он принимает запросы на
естественном языке, помогает подбирать товары, уточняет предпочтения и
возвращает рекомендации с карточками товаров.

Подробный дизайн описан в `server/ai-service/DESIGN.md`.

## Что делает сервис

`ai-service` работает как отдельный FastAPI-сервис, который:

- принимает чат-запросы от frontend через SSE;
- использует OpenAI API для генерации ответа, tool use и embeddings;
- ходит в `product-service` за товарами, фильтрами, фасетами и semantic search;
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
      -> OpenAI API
      -> product-service
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
`product-service`. В product DB должна появиться таблица `product_embeddings`
с `pgvector`, а `ai-service` будет только эмбеддить пользовательский запрос и
отправлять query vector во внутренний endpoint product-service.

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
- product embeddings worker и semantic endpoint ещё нужно добавить в
  `product-service`.
