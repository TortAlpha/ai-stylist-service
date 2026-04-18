-- ============================================================
-- Product Service — PostgreSQL Init Script (v8)
-- Branded second-hand clothing store
-- ============================================================
--
-- Top-level bootstrap file for Docker/Postgres init.
-- Schema is split into logical fragments under migrations/init/.
-- Seed scripts are executed directly below in a dedicated transaction block.

BEGIN;

\ir init/00_extensions.sql
\ir init/01_lookup_tables.sql
\ir init/02_product_schema.sql
\ir init/03_functions.sql
\ir init/04_triggers.sql
\ir init/05_views.sql
\ir init/06_embedding.sql
\ir init/07_jobs.sql

COMMIT;

BEGIN;

\ir seeds/brand_seed.sql
\ir seeds/category_seed.sql
\ir seeds/tags_seed.sql
\ir seeds/purchase_location_seed.sql

COMMIT;
