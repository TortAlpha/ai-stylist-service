
# Server — Rust Backend Microservices

Parent instructions: [../CLAUDE.md](../CLAUDE.md)

## Overview

Mono-repo of Rust microservices. Each service is a separate Cargo project in its own directory with its own database, migrations, and Dockerfile.

## Services

```
server/
├── product-service/     # Product catalog, brands, categories, tags, listings, embeddings
├── auth-service/        # JWT authentication, sessions (planned)
├── user-service/        # User profiles, addresses, roles (planned)
├── order-service/       # Orders (future versions)
├── analytics-service/   # Statistics and reporting via ClickHouse (planned)
└── chat-service/        # User chat with AI agent (planned)
```

## Architecture per Service

Each service follows the same layered structure:

```
<service>/src/
├── main.rs              # Entrypoint: config, DB pool, app state, server startup
├── lib.rs               # Re-exports modules
├── config.rs            # Environment-based configuration (envy/dotenvy)
├── domain/
│   ├── mod.rs           # Domain model re-exports
│   ├── error.rs         # thiserror enums (ServiceError, RepoError)
│   ├── <entity>.rs      # Domain models (Product, Brand, Category, etc.)
│   ├── request_dto.rs   # Incoming request DTOs (Create*, Update*)
│   ├── response_dto.rs  # Outgoing response DTOs
│   ├── mappers.rs       # Domain <-> DTO mapping functions
│   └── utils.rs         # SKU generation, helpers
├── handler/
│   ├── mod.rs           # Route configuration (actix-web scope + resources)
│   └── <entity>_handler.rs  # HTTP handlers: extract request -> call service -> return response
├── service/
│   ├── mod.rs           # Service trait re-exports
│   └── <entity>_service.rs  # Business logic, calls repo traits
├── repo/
│   ├── mod.rs           # Repo module re-exports
│   ├── traits/          # Async trait definitions for each entity repo
│   └── impls/           # SQLx implementations of repo traits
└── storage/
    ├── mod.rs
    ├── traits.rs        # Storage trait (upload, delete, get_url)
    └── s3.rs            # S3/MinIO implementation
```

## Key Patterns

### Dependency Injection
All repo and storage traits are injected as `Arc<dyn Trait>` into service structs, and services are injected into handlers via Actix `web::Data`.

### Error Handling
- `RepoError` for database-level errors
- `ServiceError` wraps `RepoError` and adds business-logic errors
- `ServiceError` implements `actix_web::ResponseError` for automatic HTTP error responses

### DTOs and Mappers
- Request DTOs (`Create*`, `Update*`) live in `domain/request_dto.rs`
- Response DTOs live in `domain/response_dto.rs`
- All conversions between domain models and DTOs go through explicit functions in `domain/mappers.rs`
- Never return domain models directly from handlers

### Testing
- Feature flag `test-mocks` enables `#[automock]` on repo/storage traits
- Unit tests mock repo layer and test service logic
- Tests live in `tests/` directory of each service
- Run: `cargo test` from the service directory

## Database Schemas

### auth-service
```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    refresh_token VARCHAR(512) NOT NULL UNIQUE,
    device_type VARCHAR(50) NOT NULL,
    last_activity_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### user-service
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    surname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    phone_number VARCHAR(255) UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE addresses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    street VARCHAR(255) NOT NULL,
    building_num VARCHAR(20) NOT NULL,
    floor_num INTEGER,
    apartment_num VARCHAR(20),
    post_index VARCHAR(20) NOT NULL
);
```

### product-service
See `product-service/migrations/init.sql` for the full schema (brands, categories, tags, products, listings, product_details, etc.)

## Development

```bash
# From any service directory:
cargo check          # Type-check
cargo build          # Compile
cargo run            # Run the service
cargo test           # Run tests

# Environment setup
cp .env.example .env # Then fill in DATABASE_URL, SERVICE_PORT, S3 buckets
```

## Conventions

- Rust edition 2024
- `thiserror` for error enums
- `rust_decimal::Decimal` for prices
- `uuid::Uuid` for IDs, `chrono::DateTime<Utc>` for timestamps
- `async_trait` on all trait definitions
- Migrations in `migrations/init.sql` per service
- One PostgreSQL database per service (except analytics-service which uses ClickHouse)
- gRPC (tonic) for interservice communication
- Kafka (rdkafka) for async messaging
