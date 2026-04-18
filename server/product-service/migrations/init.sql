-- ============================================================
-- Product Service — PostgreSQL Init Script (v8)
-- Branded second-hand clothing store
-- ============================================================
--
-- Top-level bootstrap file for Docker/Postgres init.
-- Schema is split into logical fragments under migrations/init/.
-- Optional seed scripts live under migrations/seeds/ and are not auto-run.

BEGIN;

\ir init/00_extensions.sql
\ir init/01_lookup_tables.sql
\ir init/02_product_schema.sql
\ir init/03_functions.sql
\ir init/04_triggers.sql
\ir init/05_views.sql
\ir init/06_embedding.sql

COMMIT;
