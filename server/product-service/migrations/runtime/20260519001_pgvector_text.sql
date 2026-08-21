-- Phase 0a: enable pgvector and create the product text embedding table.
--
-- Dim is fixed to 1024 to match Cohere embed-multilingual-v3.0 (chosen as
-- the multimodal provider for ai-service and product-service; see
-- server/ai-service/DESIGN.md). Changing dim later requires recreating
-- this table and full reindex.
--
-- Image-side table is added in Phase 0b.

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS product_text_embeddings (
    product_id  UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    embedding   vector(1024) NOT NULL,
    text_hash   TEXT         NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS product_text_embeddings_hnsw
    ON product_text_embeddings
    USING hnsw (embedding vector_cosine_ops);

COMMENT ON TABLE  product_text_embeddings IS
    'Per-product text embedding (Cohere embed-multilingual-v3.0, dim 1024)';
COMMENT ON COLUMN product_text_embeddings.text_hash IS
    'SHA256 of generate_product_text(product_id); skip re-embed when unchanged';
