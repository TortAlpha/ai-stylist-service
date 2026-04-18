-- ============================================================
-- MAIN PRODUCT TABLE
-- ============================================================

CREATE SEQUENCE product_sku_seq START 1;

CREATE TABLE product (
    id                UUID           PRIMARY KEY DEFAULT uuid_generate_v4(),
    sku               VARCHAR(64)    NOT NULL UNIQUE,
    name              VARCHAR(500)   NOT NULL,
    brand_id          INT            NOT NULL REFERENCES brand(id),
    category_id       INT            NOT NULL REFERENCES category(id),
    purchase_location_id INT         REFERENCES purchase_location(id),

    status            VARCHAR(20)    NOT NULL DEFAULT 'intake'
                      CHECK (status IN (
                          'intake', 'inspection', 'rejected', 'preparation',
                          'photo_queue', 'photo_done', 'ready',
                          'reserved', 'sold', 'returned'
                      )),

    purchase_price    DECIMAL(12, 2) CHECK (purchase_price > 0),
    currency          VARCHAR(3)     NOT NULL DEFAULT 'RSD',

    ai_notes          TEXT,

    images_path       TEXT,
    image_count       INT            NOT NULL DEFAULT 0 CHECK (image_count >= 0),
    preview_image_key TEXT,
    product_url       TEXT,

    version           INT            NOT NULL DEFAULT 1,

    is_deleted        BOOLEAN        NOT NULL DEFAULT false,
    deleted_at        TIMESTAMPTZ,
    created_at        TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_brand       ON product (brand_id);
CREATE INDEX idx_product_category    ON product (category_id);
CREATE INDEX idx_product_purchase_location ON product (purchase_location_id);
CREATE INDEX idx_product_status      ON product (status);
CREATE INDEX idx_product_not_deleted ON product (is_deleted) WHERE is_deleted = false;

COMMENT ON TABLE  product                    IS 'Core product table — one row per unique item in the store';
COMMENT ON COLUMN product.status             IS 'Lifecycle status: intake → inspection → … → ready → reserved/sold/returned';
COMMENT ON COLUMN product.purchase_price     IS 'How much the item was bought for (cost basis)';
COMMENT ON COLUMN product.purchase_location_id IS 'FK to normalized purchase_location dimension for sourcing filters and analytics';
COMMENT ON COLUMN product.ai_notes           IS 'AI-generated notes — descriptions, tags suggestions, pricing hints, etc.';
COMMENT ON COLUMN product.images_path        IS 'Path to image directory in storage (e.g. S3 bucket prefix)';
COMMENT ON COLUMN product.image_count        IS 'Number of images uploaded — used for additive uploads';
COMMENT ON COLUMN product.preview_image_key  IS 'S3 object key for preview thumbnail — presigned URL generated on read';
COMMENT ON COLUMN product.product_url        IS 'Link to the full product page on the storefront';
COMMENT ON COLUMN product.version            IS 'Optimistic lock counter — incremented on every update';

-- ============================================================
-- PRODUCT DETAILS (unified)
-- ============================================================

CREATE TABLE product_details (
    product_id         UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,

    -- Common
    material           VARCHAR(200),
    condition          VARCHAR(30) NOT NULL CHECK (
                           condition IN ('new_with_tags', 'excellent', 'good', 'fair')
                       ),
    color              VARCHAR(100),
    year_of_release    INT CHECK (year_of_release BETWEEN 1900 AND 2100),
    is_vintage         BOOLEAN NOT NULL DEFAULT false,
    is_collab          BOOLEAN NOT NULL DEFAULT false,
    collab_name        VARCHAR(300),
    is_limited_edition BOOLEAN NOT NULL DEFAULT false,
    special_notes      TEXT,

    -- ============================================================
    -- Unified sizing fields (interpretation depends on category.size_group)
    -- ============================================================
    --
    -- size_group       | size_value        | size_value2  | size_system | shoe_width
    -- -----------------+-------------------+--------------+-------------+-----------
    -- letter           | XS/S/M/L/XL/XXL  |              |             |
    -- letter_or_numeric| S/M/L OR 36/38/40 |              | EU/US/UK/IT/FR (if numeric) |
    -- waist_length     | 28/30/32 (waist)  | 30/32 (length)|            |
    -- shoe             | 42/9/8 (size)     |              | EU/US/UK    | narrow/regular/wide
    -- ring             | 7/54/N (size)     |              | US/EU/UK    |
    -- measurement_cm   | (use measurement_cm column)      |             |
    -- hat              | S/M/L OR (use measurement_cm)    |             |
    -- dimensions       | (use w/h/d columns)|             |             |
    -- one_size         | (no sizing)       |              |             |
    -- -----------------+-------------------+--------------+-------------+-----------

    size_value         VARCHAR(20),
    size_value2        VARCHAR(20),
    size_system        VARCHAR(5) CHECK (size_system IN ('EU', 'US', 'UK', 'IT', 'FR')),
    measurement_cm     DECIMAL(6, 1),

    -- Clothing
    fit                VARCHAR(20) CHECK (fit IN ('regular', 'slim', 'oversized', 'relaxed')),

    -- Footwear
    shoe_width         VARCHAR(10) CHECK (shoe_width IN ('narrow', 'regular', 'wide')),
    insole_length_cm   DECIMAL(4, 1),

    -- Bags
    width_cm           DECIMAL(6, 1),
    height_cm          DECIMAL(6, 1),
    depth_cm           DECIMAL(6, 1),
    handle_type        VARCHAR(50) CHECK (handle_type IN ('shoulder', 'crossbody', 'hand', 'backpack', 'tote')),
    bag_size_label     VARCHAR(20),

    -- Jewelry
    metal              VARCHAR(100),
    stone              VARCHAR(100),
    clasp_type         VARCHAR(50)
);

CREATE INDEX idx_details_condition  ON product_details (condition);
CREATE INDEX idx_details_size_value ON product_details (size_value)  WHERE size_value IS NOT NULL;
CREATE INDEX idx_details_bag_width  ON product_details (width_cm)    WHERE width_cm IS NOT NULL;

COMMENT ON TABLE  product_details                  IS 'All product attributes — common + type-specific in one table';
COMMENT ON COLUMN product_details.condition        IS 'Item condition: new_with_tags, excellent, good, fair';
COMMENT ON COLUMN product_details.collab_name      IS 'Collaboration name if applicable (e.g. Nike x Off-White)';
COMMENT ON COLUMN product_details.size_value       IS 'Primary size value — interpretation depends on category.size_group (e.g. "M", "42", "28")';
COMMENT ON COLUMN product_details.size_value2      IS 'Secondary size value — e.g. inseam length for jeans ("32" in W28/L32)';
COMMENT ON COLUMN product_details.size_system      IS 'Sizing system: EU, US, UK, IT, FR — used with letter_or_numeric, shoe, ring groups';
COMMENT ON COLUMN product_details.measurement_cm   IS 'Direct measurement in cm — belt length, bracelet circumference, necklace chain, hat circumference';
COMMENT ON COLUMN product_details.fit              IS 'Clothing fit: regular, slim, oversized, relaxed';
COMMENT ON COLUMN product_details.shoe_width       IS 'Footwear width: narrow, regular, wide';
COMMENT ON COLUMN product_details.insole_length_cm IS 'Footwear insole length in cm';
COMMENT ON COLUMN product_details.handle_type      IS 'Bag handle/carry style';
COMMENT ON COLUMN product_details.bag_size_label   IS 'Brand-specific bag size designation (e.g. PM, MM, GM, Mini, Small, Medium, Large)';
COMMENT ON COLUMN product_details.metal            IS 'Jewelry metal type (gold, silver, etc.)';
COMMENT ON COLUMN product_details.stone            IS 'Jewelry stone type (diamond, ruby, etc.)';

-- ============================================================
-- JUNCTION TABLES
-- ============================================================

CREATE TABLE product_style_tag (
    product_id   UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    style_tag_id INT  NOT NULL REFERENCES style_tag(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, style_tag_id)
);

COMMENT ON TABLE product_style_tag IS 'Junction: product <-> style tags (for embeddings)';

CREATE TABLE product_vibe_tag (
    product_id  UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    vibe_tag_id INT  NOT NULL REFERENCES vibe_tag(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, vibe_tag_id)
);

COMMENT ON TABLE product_vibe_tag IS 'Junction: product <-> vibe tags (for embeddings)';

CREATE TABLE product_season (
    product_id UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    season_id  INT  NOT NULL REFERENCES season(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, season_id)
);

COMMENT ON TABLE product_season IS 'Junction: product <-> seasons (for embeddings)';

-- ============================================================
-- SIMILAR PRODUCTS (for embeddings)
-- ============================================================

CREATE TABLE similar_products (
    product_id         UUID  NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    similar_product_id UUID  NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    similarity_score   FLOAT NOT NULL CHECK (similarity_score BETWEEN 0 AND 1),
    PRIMARY KEY (product_id, similar_product_id),
    CHECK (product_id != similar_product_id)
);

CREATE INDEX idx_similar_products_target ON similar_products (similar_product_id);
CREATE UNIQUE INDEX uq_similar_products_pair_undirected
    ON similar_products (
        LEAST(product_id, similar_product_id),
        GREATEST(product_id, similar_product_id)
    );

COMMENT ON TABLE  similar_products                  IS 'Pre-computed product similarity pairs for recommendations (embedding cosine similarity)';
COMMENT ON COLUMN similar_products.similarity_score IS 'Cosine similarity between product embeddings (0..1)';

-- ============================================================
-- PRODUCT HISTORY (audit log — JSONB diff)
-- ============================================================

CREATE TABLE product_history (
    id          BIGSERIAL    PRIMARY KEY,
    product_id  UUID         NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    version     INT          NOT NULL,
    changes     JSONB        NOT NULL,
    changed_by  VARCHAR(200),
    changed_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_history_product ON product_history (product_id, version);
CREATE INDEX idx_product_history_changes ON product_history USING GIN (changes);

COMMENT ON TABLE  product_history            IS 'Audit log for the full product aggregate — product row, details row, and tag/season links';
COMMENT ON COLUMN product_history.changes    IS '{"event": "created", "product": {...}, "details": {...}, "style_tags": {"added": [...], "removed": [...]}, ...}';
COMMENT ON COLUMN product_history.changed_by IS 'Who made the aggregate change — set via SET LOCAL app.changed_by inside the write transaction';

CREATE TABLE product_audit_buffer (
    txid              BIGINT        NOT NULL,
    product_id        UUID          NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    created_in_tx     BOOLEAN       NOT NULL DEFAULT false,
    product_changes   JSONB         NOT NULL DEFAULT '{}'::jsonb,
    detail_changes    JSONB         NOT NULL DEFAULT '{}'::jsonb,
    style_tag_added   INT[]         NOT NULL DEFAULT '{}',
    style_tag_removed INT[]         NOT NULL DEFAULT '{}',
    vibe_tag_added    INT[]         NOT NULL DEFAULT '{}',
    vibe_tag_removed  INT[]         NOT NULL DEFAULT '{}',
    season_added      INT[]         NOT NULL DEFAULT '{}',
    season_removed    INT[]         NOT NULL DEFAULT '{}',
    changed_by        VARCHAR(200),
    changed_at        TIMESTAMPTZ   NOT NULL DEFAULT now(),
    PRIMARY KEY (txid, product_id)
);

COMMENT ON TABLE product_audit_buffer IS 'Internal helper table for deferred aggregate audit; should normally stay empty outside active transactions';

-- ============================================================
-- ANALYTICS SUPPORT
-- ============================================================

CREATE TABLE product_status_history (
    id              BIGSERIAL    PRIMARY KEY,
    product_id      UUID         NOT NULL,
    product_version INT          NOT NULL CHECK (product_version > 0),
    old_status      VARCHAR(20)
                    CHECK (old_status IS NULL OR old_status IN (
                        'intake', 'inspection', 'rejected', 'preparation',
                        'photo_queue', 'photo_done', 'ready',
                        'reserved', 'sold', 'returned'
                    )),
    new_status      VARCHAR(20)  NOT NULL
                    CHECK (new_status IN (
                        'intake', 'inspection', 'rejected', 'preparation',
                        'photo_queue', 'photo_done', 'ready',
                        'reserved', 'sold', 'returned'
                    )),
    changed_by      VARCHAR(200),
    changed_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_status_history_product ON product_status_history (product_id, changed_at);
CREATE INDEX idx_product_status_history_new_status ON product_status_history (new_status);

COMMENT ON TABLE product_status_history IS 'Structured product status transitions for operational analytics and lead-time calculations';
COMMENT ON COLUMN product_status_history.product_version IS 'Aggregate version visible after the transition was applied';
COMMENT ON COLUMN product_status_history.old_status IS 'Previous lifecycle status (NULL on create)';
COMMENT ON COLUMN product_status_history.new_status IS 'New lifecycle status';

CREATE TABLE product_outbox_event (
    id              BIGSERIAL    PRIMARY KEY,
    product_id      UUID         NOT NULL,
    event_type      VARCHAR(40)  NOT NULL
                    CHECK (event_type IN (
                        'product.created',
                        'product.updated',
                        'product.status_changed',
                        'product.deleted'
                    )),
    product_version INT          NOT NULL CHECK (product_version > 0),
    payload         JSONB        NOT NULL,
    occurred_at     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ,
    retry_count     INT          NOT NULL DEFAULT 0 CHECK (retry_count >= 0)
);

CREATE UNIQUE INDEX uq_product_outbox_event_version_type
    ON product_outbox_event (product_id, product_version, event_type);
CREATE INDEX idx_product_outbox_unpublished
    ON product_outbox_event (occurred_at, id)
    WHERE published_at IS NULL;
CREATE INDEX idx_product_outbox_product
    ON product_outbox_event (product_id, occurred_at);

COMMENT ON TABLE product_outbox_event IS 'Transactional outbox for downstream consumers such as analytics-service';
COMMENT ON COLUMN product_outbox_event.event_type IS 'Lifecycle event published by product-service: created, updated, status_changed, deleted';
COMMENT ON COLUMN product_outbox_event.payload IS 'Event envelope with lifecycle change payload and analytics snapshot';
COMMENT ON COLUMN product_outbox_event.product_version IS 'Aggregate version visible to consumers for this lifecycle event';
