# SC — Branded Second-Hand Clothing Store

## Project Overview

E-commerce platform for selling branded second-hand clothing. Initially — an inventory management system (buy/sell and tracking intermediate statuses). Microservice architecture with a Rust backend and Angular TypeScript frontend.

## Current Version — v1

v1 focuses on core product management and basic role-based functionality:

- **product-service**: full CRUD for products, brands, categories, tags, listings; image upload to S3; product versioning and status tracking
- **auth-service**: JWT authentication, session management
- **user-service**: user registration, profiles, addresses, roles (admin / user)
- **Notifications**: basic notification system (email/in-app) for product status changes and admin actions
- **Client (Angular)**: admin panel for product management; public catalog with filters; auth flow (login/register)

**Not in v1** (planned for later versions): order-service, chat-service, RAG/AI agent, analytics-service (ClickHouse), Kafka messaging

### User Roles

- **Admin**: view products, add/edit products, view product change history, view general and specific statistics
- **Authorized user**: view products, search via filters, ask questions in chat (AI agent), place orders
- **Unauthorized user**: view products and general information only

### Product Domain

Products include clothing, footwear, and accessories. Each product has details, sizes, etc. Chat communication with authorized users goes through an AI agent (RAG), and the product-service stores embeddings for vector search.

## Repository Structure

```
sc/
├── server/                  # Backend microservices (Rust)
│   └── product-service/     # Product catalog, brands, categories, tags, listings
├── client/                  # Frontend application (Angular)
└── shared/                  # Shared code/contracts between services
```

Each subdirectory (`server/`, `client/`) has its own `CLAUDE.md` with area-specific instructions. **Always read the relevant child CLAUDE.md before working in that area.**

## Architecture Principles

- **Microservices**: each service owns its database and exposes a REST API
- **Layered architecture** in each service: `handler` -> `service` -> `repo` (repository pattern)
- Traits for repository and storage abstractions — implementations are injected via `Arc<dyn Trait>`
- DTOs (`request_dto`, `response_dto`) are separate from domain models; mapping goes through explicit mapper functions in `domain/mappers.rs`
- Interservice communication via gRPC
- API Gateway: nginx

## Microservices

| Service | Purpose | DB | Version |
|---|---|---|---|
| `product-service` | Product catalog, brands, categories, tags, listings, embeddings | PostgreSQL | **v1** |
| `auth-service` | Authentication, JWT tokens, sessions | PostgreSQL | **v1** |
| `user-service` | User profiles, addresses, roles | PostgreSQL | **v1** |
| `order-service` | Orders | PostgreSQL | v2+ |
| `analytics-service` | Statistics and reporting | ClickHouse | v2+ |
| `chat-service` | User chat with AI agent | PostgreSQL | v2+ |
| RAG service | LangGraph system + FastAPI | — | v2+ |

## Tech Stack

### Backend (Rust)
- Web Framework: Actix-web
- Database: SQLx (async PostgreSQL driver)
- Authentication: jsonwebtoken (JWT)
- Message Broker: rdkafka (Kafka client)
- Image Processing: image, webp crates
- Object Storage: aws-sdk-s3 or rust-s3
- Email: lettre
- Serialization: serde, serde_json
- Async Runtime: tokio
- HTTP Client: reqwest
- Validation: validator
- Logging: tracing, tracing-subscriber
- Interservice: tonic (gRPC)
- OpenAPI: utoipa + utoipa-swagger-ui

### Frontend (Angular)
- Framework: Angular 17+
- State Management: NgRx or Akita
- UI Components: Angular Material or PrimeNG
- HTTP Client: Angular HttpClient
- Forms: Reactive Forms
- Routing: Angular Router
- Authentication: JWT interceptor

### Infrastructure
- Containerization: Docker, Docker Compose
- Message Broker: Apache Kafka + Zookeeper
- Databases: PostgreSQL (one instance per service), ClickHouse (analytics)
- Object Storage: MinIO (S3-compatible)
- API Gateway: nginx

## Conventions

### Rust
- Edition 2024
- Use `thiserror` style enums for errors (see `domain/error.rs`)
- Prices are `rust_decimal::Decimal`, never floating point
- IDs are `uuid::Uuid`, timestamps are `chrono::DateTime<Utc>`
- Feature flag `test-mocks` enables mockall trait mocking for unit tests
- Async everywhere — all repo/service traits are `#[async_trait]`

### TypeScript
- camelCase for variables and functions

### Database
- Migrations live in `migrations/init.sql` per service
- Lookup tables: brand, category, tags, seasons, statuses
- Products have versioning, statuses, and reservation support
- SKU generation uses brand code + category code

### API
- All routes under `/api` scope
- RESTful naming: `/api/products`, `/api/brands`, `/api/categories`, `/api/tags`, `/api/listings`
- Request/response bodies are JSON; multipart for image uploads

### Git
- Branch naming: `feature/<name>`, `fix/<name>`
- Commit messages: `feat:`, `fix:`, `refactor:`, `test:`, `docs:` prefixes
- Main branch: `master`

## Development Commands

```bash
# Build & check (from service directory)
cargo check
cargo build

# Run
cp .env.example .env    # fill in DATABASE_URL, SERVICE_PORT, S3 buckets
cargo run

# Tests
cargo test
cargo test test_name    # specific test
```

## Environment Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `SERVICE_PORT` | HTTP port for the service |
| `PRODUCT_IMAGES_BUCKET` | S3 bucket for full images |
| `PRODUCT_PREVIEW_BUCKET` | S3 bucket for previews |

## Multi-Agent Workflow

This project is developed using Claude Code's multi-agent system. Each agent working in a subdirectory should:

1. Read the local `CLAUDE.md` before starting work
2. Run `cargo check` after any code changes in Rust services
3. Run `cargo test` after completing a feature or fix
4. Keep domain models, DTOs, and mappers in sync when modifying data structures
