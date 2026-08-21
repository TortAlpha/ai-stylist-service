-- Phase 0b: per-image embeddings table.
--
-- One row per product image (image_idx matches the S3 layout
-- products/{product_id}/{image_idx}/{thumb,medium,full}.webp). Vectors
-- share the same space as product_text_embeddings (Cohere
-- embed-multilingual-v3.0 multimodal, dim 1024).
--
-- Hybrid scoring (per_image): the query vector is compared against every
-- image row and the maximum cosine similarity per product wins —
-- enables retrieval where one good camera angle is enough.

CREATE TABLE IF NOT EXISTS product_image_embeddings (
    product_id  UUID         NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    image_idx   INT          NOT NULL,
    embedding   vector(1024) NOT NULL,
    image_hash  TEXT         NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (product_id, image_idx)
);

CREATE INDEX IF NOT EXISTS product_image_embeddings_hnsw
    ON product_image_embeddings
    USING hnsw (embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS product_image_embeddings_product
    ON product_image_embeddings (product_id);

COMMENT ON TABLE  product_image_embeddings IS
    'Per-image embedding (Cohere embed-multilingual-v3.0 image mode, dim 1024)';
COMMENT ON COLUMN product_image_embeddings.image_idx IS
    'S3 image index matching products/{product_id}/{image_idx}/medium.webp';
COMMENT ON COLUMN product_image_embeddings.image_hash IS
    'SHA256 of the medium.webp bytes — skip re-embed when unchanged';
