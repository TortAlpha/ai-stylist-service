# Product DB initialization

For now this project uses bootstrap-only database initialization:

1. `init.sql` + `init/*.sql`
   - Executed by Docker Postgres `initdb` on first startup of a fresh volume.
   - Contains the full current schema (tables, functions, triggers, views).
2. `seeds/*.sql`
   - Optional data scripts, executed manually when needed.

## Operational rule

- Keep `init/*.sql` as the single source of truth for schema design.
- When schema changes in dev, update `init/*.sql` directly.
- If/when runtime migrations are introduced later, document that transition here.

## SQL tests

Run invariant tests after schema changes:

```bash
server/product-service/tests/sql/run_sql_tests.sh
```
