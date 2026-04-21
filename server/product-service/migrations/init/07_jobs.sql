-- ============================================================
-- JOBS QUEUE (lightweight pgqueue)
-- ============================================================
-- Generic background job queue used to atomically schedule work
-- alongside business writes (e.g. enqueue an S3 upload in the same
-- transaction as a product insert).
--
-- Workers pick rows up with FOR UPDATE SKIP LOCKED so multiple
-- workers can run in parallel without blocking each other.

CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    kind          TEXT        NOT NULL,
    payload       JSONB       NOT NULL DEFAULT '{}'::jsonb,

    -- Binary blobs (e.g. raw image bytes) live in dedicated columns
    -- instead of payload to keep JSONB scans cheap.
    blobs         BYTEA[],
    blob_types    TEXT[],

    status        TEXT        NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending', 'failed')),
    attempts      INT         NOT NULL DEFAULT 0,
    max_attempts  INT         NOT NULL DEFAULT 5,
    last_error    TEXT,

    run_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT jobs_blobs_pairing CHECK (
        (blobs IS NULL AND blob_types IS NULL)
        OR (
            blobs IS NOT NULL AND blob_types IS NOT NULL
            AND cardinality(blobs) = cardinality(blob_types)
        )
    )
);

CREATE INDEX idx_jobs_pending ON jobs (run_at) WHERE status = 'pending';
CREATE INDEX idx_jobs_kind    ON jobs (kind);

COMMENT ON TABLE  jobs           IS 'Background job queue — workers poll with FOR UPDATE SKIP LOCKED';
COMMENT ON COLUMN jobs.kind      IS 'Discriminator for the worker handler (e.g. "upload_product_images")';
COMMENT ON COLUMN jobs.payload   IS 'JSON metadata for the job (ids, options)';
COMMENT ON COLUMN jobs.blobs     IS 'Optional raw bytes (e.g. file contents) — paired by index with blob_types';
COMMENT ON COLUMN jobs.status    IS 'pending = picked up by next free worker; failed = exhausted retries';
COMMENT ON COLUMN jobs.run_at    IS 'Earliest time the job may run — used for retry backoff';
