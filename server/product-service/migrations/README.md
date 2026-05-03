# Product DB initialization & migrations

The product service uses **two layers** of SQL:

1. **Bootstrap** — `init.sql` + `init/*.sql` + `seeds/*.sql`
   Executed by Docker Postgres `initdb` on the **first startup of a fresh volume only**
   (Postgres skips initdb if `/var/lib/postgresql/data` is non-empty). Source of
   truth for the initial schema and lookup data.

2. **Runtime migrations** — `runtime/*.sql`
   Forward-only schema/data deltas applied automatically by the service at
   startup via `sqlx::migrate!("./migrations/runtime")` (see
   `src/main.rs`). sqlx tracks applied migrations in the `_sqlx_migrations`
   table and refuses to re-run or alter them.

## Operational rules

- **Bootstrap (`init/*.sql`, `seeds/*.sql`) — single source of truth for fresh volumes.**
  When schema or seed data changes, edit these files **and** ship a runtime migration.
  This duplication is the cost of init-only bootstrap; without it a fresh dev DB
  would lag behind whatever's been migrated elsewhere.

- **Runtime migrations — forward-only, idempotent, transactional, immutable.**
  - Filename: `<VERSION>_<short_slug>.sql` where `<VERSION>` is a single token
    before the first underscore (e.g. `20260502001_seasons_rename.sql`).
    sqlx parses the version as `filename.split_once('_')`; two migrations with
    the same prefix collide.
  - Do **not** add manual `BEGIN; … COMMIT;` statements. Postgres migrations
    run via `sqlx::migrate!()` are already transactional; write the migration
    body directly unless a statement specifically requires non-transactional
    execution.
  - Defensive SQL only: `IF NOT EXISTS`, `IF EXISTS`, `ON CONFLICT DO NOTHING`,
    `UPDATE … WHERE name = …`. Re-running the same body must be a no-op so
    that bootstrap state and runtime-applied state converge.
  - **Never edit a migration file after it has been applied anywhere.** sqlx
    stores a checksum and the service will refuse to start if the file changed.
    Add a new forward-migration instead.
  - No down-migrations.

- **Auto-apply on service startup.**
  `product-service` calls `sqlx::migrate!("./migrations/runtime").run(&pool)` after
  connecting to Postgres. If a migration fails the service does not start —
  this is intentional. Investigate, write a forward-fix migration, redeploy.

- **First run on an existing snapshot.**
  When this service first starts against a Postgres that pre-dates runtime
  migrations, sqlx finds no `_sqlx_migrations` table, creates it, and applies
  every `runtime/*.sql` in order. Because each migration is idempotent, this
  succeeds even if the schema/seed changes were already applied manually
  earlier.

## Deploy ordering

Service deploy and the migrations it carries are bundled by definition (the
binary embeds the migration files via `include_str!`). Practical implications:

- **Additive migrations** (new column nullable, new value, new table): safe
  to deploy with the code that uses them.
- **Destructive migrations** (drop column, narrow CHECK): split into two
  releases — first migration that removes code dependency, then migration
  that removes the column.

## Local development

- Fresh volume: bootstrap runs, then service start applies all runtime
  migrations on top (no-ops, since bootstrap is mirrored).
- Existing volume / snapshot: only runtime migrations apply on service start.

To wipe and reapply everything from scratch in compose:
```bash
docker compose down -v        # drop the volume
docker compose --profile local-db up
```

## SQL tests

Run invariant tests after schema changes:

```bash
server/product-service/tests/sql/run_sql_tests.sh
```
