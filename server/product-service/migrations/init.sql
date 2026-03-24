-- ============================================================
-- Product Service — PostgreSQL Init Script (v4)
-- Branded second-hand clothing store
-- ============================================================

BEGIN;

-- ============================================================
-- EXTENSIONS
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================
-- LOOKUP TABLES
-- ============================================================

-- ------- Brands -------

CREATE TABLE brand (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(200) NOT NULL UNIQUE,
    code        VARCHAR(10)  NOT NULL UNIQUE,
    tier        VARCHAR(20)  NOT NULL CHECK (tier IN ('mass', 'premium', 'luxury')),
    country     VARCHAR(100),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE  brand      IS 'Brand directory';
COMMENT ON COLUMN brand.code IS 'Short uppercase code for SKU generation (e.g. NK, AD, GU)';
COMMENT ON COLUMN brand.tier IS 'Market segment: mass-market, premium, luxury';

-- ------- Categories (hierarchical) -------

CREATE TABLE category (
    id           SERIAL PRIMARY KEY,
    name         VARCHAR(200) NOT NULL,
    code         VARCHAR(10)  NOT NULL,
    parent_id    INT          REFERENCES category(id),
    gender       VARCHAR(10)  NOT NULL CHECK (gender IN ('male', 'female', 'unisex')),
    product_type VARCHAR(20)  NOT NULL
                 CHECK (product_type IN ('clothing', 'footwear', 'bags', 'jewelry', 'accessories')),
    UNIQUE (name, parent_id, gender)
);

CREATE INDEX idx_category_parent       ON category (parent_id);
CREATE INDEX idx_category_product_type ON category (product_type);

COMMENT ON TABLE  category                IS 'Hierarchical product categories (self-referencing tree)';
COMMENT ON COLUMN category.code           IS 'Short uppercase code for SKU generation (e.g. SNK, JGR, PFJ)';
COMMENT ON COLUMN category.parent_id      IS 'Parent category FK (NULL = root category)';
COMMENT ON COLUMN category.product_type   IS 'Product type — determines which detail fields are relevant (inherited from root category)';

-- ------- Style tags (for embeddings) -------

CREATE TABLE style_tag (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

COMMENT ON TABLE style_tag IS 'Style classification tags for embeddings (streetwear, formal, casual, etc.)';

-- ------- Vibe tags (for embeddings) -------

CREATE TABLE vibe_tag (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

COMMENT ON TABLE vibe_tag IS 'Mood / vibe tags for embeddings (hype, minimalism, retro, etc.)';

-- ------- Seasons (for embeddings) -------

CREATE TABLE season (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

COMMENT ON TABLE season IS 'Season applicability for products (used in embedding pipeline)';

INSERT INTO season (name) VALUES
    ('summer'), ('winter'), ('demi-season'), ('all-season');

-- ============================================================
-- MAIN PRODUCT TABLE
-- ============================================================

CREATE SEQUENCE product_sku_seq START 1;

CREATE TABLE product (
    id                UUID           PRIMARY KEY DEFAULT uuid_generate_v4(),
    sku               VARCHAR(30)    NOT NULL UNIQUE,
    name              VARCHAR(500)   NOT NULL,
    brand_id          INT            NOT NULL REFERENCES brand(id),
    category_id       INT            NOT NULL REFERENCES category(id),

    status            VARCHAR(20)    NOT NULL DEFAULT 'intake'
                      CHECK (status IN (
                          'intake', 'inspection', 'rejected', 'preparation',
                          'photo_queue', 'photo_done', 'ready',
                          'reserved', 'sold', 'returned'
                      )),

    purchase_price    DECIMAL(12, 2) CHECK (purchase_price > 0),
    purchase_location VARCHAR(500),
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
CREATE INDEX idx_product_status      ON product (status);
CREATE INDEX idx_product_not_deleted ON product (is_deleted) WHERE is_deleted = false;

COMMENT ON TABLE  product                    IS 'Core product table — one row per unique item in the store';
COMMENT ON COLUMN product.status             IS 'Lifecycle status: intake → inspection → … → ready → reserved/sold/returned';
COMMENT ON COLUMN product.purchase_price     IS 'How much the item was bought for (cost basis)';
COMMENT ON COLUMN product.purchase_location  IS 'Where the item was sourced from (store, market, city, online, etc.)';
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

    -- Clothing
    size               VARCHAR(30),
    fit                VARCHAR(20) CHECK (fit IN ('regular', 'slim', 'oversized', 'relaxed')),

    -- Footwear
    shoe_size          VARCHAR(10),
    size_system        VARCHAR(5) CHECK (size_system IN ('EU', 'US', 'UK')),
    insole_length_cm   DECIMAL(4, 1),

    -- Bags
    width_cm           DECIMAL(6, 1),
    height_cm          DECIMAL(6, 1),
    depth_cm           DECIMAL(6, 1),
    handle_type        VARCHAR(50) CHECK (handle_type IN ('shoulder', 'crossbody', 'hand', 'backpack', 'tote')),

    -- Jewelry
    metal              VARCHAR(100),
    stone              VARCHAR(100),
    clasp_type         VARCHAR(50)
);

CREATE INDEX idx_details_condition     ON product_details (condition);
CREATE INDEX idx_details_clothing_size ON product_details (size)      WHERE size IS NOT NULL;
CREATE INDEX idx_details_shoe_size     ON product_details (shoe_size) WHERE shoe_size IS NOT NULL;
CREATE INDEX idx_details_bag_width     ON product_details (width_cm)  WHERE width_cm IS NOT NULL;

COMMENT ON TABLE  product_details              IS 'All product attributes — common + type-specific in one table';
COMMENT ON COLUMN product_details.condition    IS 'Item condition: new_with_tags, excellent, good, fair';
COMMENT ON COLUMN product_details.collab_name  IS 'Collaboration name if applicable (e.g. Nike x Off-White)';
COMMENT ON COLUMN product_details.size         IS 'Clothing size (S, M, L, XL, etc.)';
COMMENT ON COLUMN product_details.fit          IS 'Clothing fit: regular, slim, oversized, relaxed';
COMMENT ON COLUMN product_details.shoe_size    IS 'Footwear size (numeric)';
COMMENT ON COLUMN product_details.size_system  IS 'Footwear sizing system: EU, US, UK';
COMMENT ON COLUMN product_details.insole_length_cm IS 'Foot wear lenth of insole in cm';
COMMENT ON COLUMN product_details.handle_type  IS 'Bag handle/carry style';
COMMENT ON COLUMN product_details.metal        IS 'Jewelry metal type (gold, silver, etc.)';
COMMENT ON COLUMN product_details.stone        IS 'Jewelry stone type (diamond, ruby, etc.)';

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

COMMENT ON TABLE  product_history            IS 'Audit log — stores only changed fields as JSONB diff per version';
COMMENT ON COLUMN product_history.changes    IS '{"field": {"old": ..., "new": ...}}';
COMMENT ON COLUMN product_history.changed_by IS 'Who made the change — set via SET LOCAL app.changed_by before UPDATE';

-- ============================================================
-- TRIGGER FUNCTIONS
-- ============================================================

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generate SKU: AVA-{BRAND}-{GENDER}-{CATEGORY}-{SEQ}
CREATE OR REPLACE FUNCTION generate_product_sku()
RETURNS TRIGGER AS $$
DECLARE
    v_brand_code    VARCHAR(10);
    v_category_code VARCHAR(10);
    v_gender_code   VARCHAR(1);
    v_seq           BIGINT;
BEGIN
    SELECT code INTO v_brand_code
    FROM brand WHERE id = NEW.brand_id;

    SELECT code, CASE gender
        WHEN 'male'   THEN 'M'
        WHEN 'female' THEN 'F'
        WHEN 'unisex' THEN 'U'
    END
    INTO v_category_code, v_gender_code
    FROM category WHERE id = NEW.category_id;

    v_seq := nextval('product_sku_seq');

    NEW.sku = 'AVA-' || v_brand_code || '-' || v_gender_code || '-' || v_category_code || '-' || LPAD(v_seq::TEXT, 5, '0');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Version tracking + JSONB diff
CREATE OR REPLACE FUNCTION track_product_version()
RETURNS TRIGGER AS $$
DECLARE
    v_changes JSONB := '{}';
BEGIN
    IF OLD.name IS DISTINCT FROM NEW.name THEN
        v_changes := v_changes || jsonb_build_object('name',
            jsonb_build_object('old', OLD.name, 'new', NEW.name));
    END IF;

    IF OLD.brand_id IS DISTINCT FROM NEW.brand_id THEN
        v_changes := v_changes || jsonb_build_object('brand_id',
            jsonb_build_object('old', OLD.brand_id, 'new', NEW.brand_id));
    END IF;

    IF OLD.category_id IS DISTINCT FROM NEW.category_id THEN
        v_changes := v_changes || jsonb_build_object('category_id',
            jsonb_build_object('old', OLD.category_id, 'new', NEW.category_id));
    END IF;

    IF OLD.status IS DISTINCT FROM NEW.status THEN
        v_changes := v_changes || jsonb_build_object('status',
            jsonb_build_object('old', OLD.status, 'new', NEW.status));
    END IF;

    IF OLD.purchase_price IS DISTINCT FROM NEW.purchase_price THEN
        v_changes := v_changes || jsonb_build_object('purchase_price',
            jsonb_build_object('old', OLD.purchase_price, 'new', NEW.purchase_price));
    END IF;

    IF OLD.purchase_location IS DISTINCT FROM NEW.purchase_location THEN
        v_changes := v_changes || jsonb_build_object('purchase_location',
            jsonb_build_object('old', OLD.purchase_location, 'new', NEW.purchase_location));
    END IF;

    IF OLD.currency IS DISTINCT FROM NEW.currency THEN
        v_changes := v_changes || jsonb_build_object('currency',
            jsonb_build_object('old', OLD.currency, 'new', NEW.currency));
    END IF;

    IF OLD.ai_notes IS DISTINCT FROM NEW.ai_notes THEN
        v_changes := v_changes || jsonb_build_object('ai_notes',
            jsonb_build_object('old', OLD.ai_notes, 'new', NEW.ai_notes));
    END IF;

    IF OLD.images_path IS DISTINCT FROM NEW.images_path THEN
        v_changes := v_changes || jsonb_build_object('images_path',
            jsonb_build_object('old', OLD.images_path, 'new', NEW.images_path));
    END IF;

    IF OLD.image_count IS DISTINCT FROM NEW.image_count THEN
        v_changes := v_changes || jsonb_build_object('image_count',
            jsonb_build_object('old', OLD.image_count, 'new', NEW.image_count));
    END IF;

    IF OLD.preview_image_key IS DISTINCT FROM NEW.preview_image_key THEN
        v_changes := v_changes || jsonb_build_object('preview_image_key',
            jsonb_build_object('old', OLD.preview_image_key, 'new', NEW.preview_image_key));
    END IF;

    IF OLD.product_url IS DISTINCT FROM NEW.product_url THEN
        v_changes := v_changes || jsonb_build_object('product_url',
            jsonb_build_object('old', OLD.product_url, 'new', NEW.product_url));
    END IF;

    IF OLD.is_deleted IS DISTINCT FROM NEW.is_deleted THEN
        v_changes := v_changes || jsonb_build_object('is_deleted',
            jsonb_build_object('old', OLD.is_deleted, 'new', NEW.is_deleted));
    END IF;

    IF v_changes = '{}' THEN
        RETURN NEW;
    END IF;

    INSERT INTO product_history (product_id, version, changes, changed_by, changed_at)
    VALUES (
        OLD.id,
        OLD.version,
        v_changes,
        COALESCE(current_setting('app.changed_by', true), current_user),
        now()
    );

    NEW.version = OLD.version + 1;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate child category inherits product_type from parent
CREATE OR REPLACE FUNCTION validate_category_product_type()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_type VARCHAR(20);
BEGIN
    IF NEW.parent_id IS NOT NULL THEN
        SELECT product_type INTO v_parent_type
        FROM category WHERE id = NEW.parent_id;

        IF v_parent_type IS DISTINCT FROM NEW.product_type THEN
            RAISE EXCEPTION 'Child category product_type (%) must match parent product_type (%)',
                NEW.product_type, v_parent_type;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- TRIGGERS
-- ============================================================

CREATE TRIGGER trg_category_product_type
    BEFORE INSERT OR UPDATE ON category
    FOR EACH ROW EXECUTE FUNCTION validate_category_product_type();

CREATE TRIGGER trg_product_sku
    BEFORE INSERT ON product
    FOR EACH ROW EXECUTE FUNCTION generate_product_sku();

CREATE TRIGGER trg_product_updated_at
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_product_version
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION track_product_version();

-- ============================================================
-- VIEW: FULL PRODUCT CARD
-- ============================================================

CREATE VIEW v_product_full AS
SELECT
    p.id,
    p.sku,
    p.name,
    p.purchase_price,
    p.purchase_location,
    p.currency,
    p.ai_notes,
    p.category_id,
    p.image_count,
    p.preview_image_key,
    p.product_url,
    p.version,

    p.status,
    c.product_type AS type,

    -- Brand
    b.id   AS brand_id,
    b.name AS brand_name,
    b.tier AS brand_tier,

    -- Category
    c.name   AS category_name,
    cp.name  AS parent_category,
    c.gender,

    -- Details
    pd.material,
    pd.condition,
    pd.color,
    pd.year_of_release,
    pd.is_vintage,
    pd.is_collab,
    pd.collab_name,
    pd.is_limited_edition,
    pd.special_notes,

    pd.size          AS clothing_size,
    pd.fit           AS clothing_fit,
    pd.shoe_size,
    pd.size_system,
    pd.insole_length_cm, 
    pd.width_cm      AS bag_width_cm,
    pd.height_cm     AS bag_height_cm,
    pd.depth_cm      AS bag_depth_cm,
    pd.handle_type   AS bag_handle_type,
    pd.metal         AS jewelry_metal,
    pd.stone         AS jewelry_stone,
    pd.clasp_type    AS jewelry_clasp_type,

    -- Tags
    COALESCE(styles.tags,  '{}') AS style_tags,
    COALESCE(vibes.tags,   '{}') AS vibe_tags,
    COALESCE(seasons.tags, '{}') AS season_tags,

    p.created_at,
    p.updated_at

FROM product p
JOIN brand     b  ON p.brand_id    = b.id
JOIN category  c  ON p.category_id = c.id
LEFT JOIN category           cp ON c.parent_id   = cp.id
LEFT JOIN product_details    pd ON p.id = pd.product_id

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(st.name ORDER BY st.name) AS tags
    FROM product_style_tag pst
    JOIN style_tag st ON pst.style_tag_id = st.id
    WHERE pst.product_id = p.id
) styles ON true

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(vt.name ORDER BY vt.name) AS tags
    FROM product_vibe_tag pvt
    JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
    WHERE pvt.product_id = p.id
) vibes ON true

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(s.name ORDER BY s.name) AS tags
    FROM product_season psn
    JOIN season s ON psn.season_id = s.id
    WHERE psn.product_id = p.id
) seasons ON true

WHERE p.is_deleted = false;

COMMENT ON VIEW v_product_full IS 'Full product card with all details and tags';

-- ============================================================
-- VIEW: STOREFRONT (chat bot & catalog)
-- ============================================================

CREATE VIEW v_product_storefront AS
SELECT * FROM v_product_full
WHERE status = 'ready';

COMMENT ON VIEW v_product_storefront IS 'Products visible to customers — only items with status ready';

-- ============================================================
-- HELPER: Generate text for embedding model
-- ============================================================

CREATE OR REPLACE FUNCTION generate_product_text(p_product_id UUID)
RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    SELECT FORMAT(
        E'%s \u2014 %s %s, %s. Type: %s.\nBrand: %s (%s). Category: %s > %s.\nStyle: %s. Vibe: %s. Season: %s.\nMaterial: %s. Condition: %s. Color: %s.\n%s%s%s%s%s',
        p.name,
        c.gender,
        COALESCE(cp.name, ''),
        c.name,
        c.product_type,
        b.name,
        b.tier,
        COALESCE(cp.name, ''),
        c.name,
        COALESCE(styles.tags, 'not specified'),
        COALESCE(vibes.tags, 'not specified'),
        COALESCE(seasons.tags, 'not specified'),
        COALESCE(pd.material, 'not specified'),
        COALESCE(pd.condition, 'not specified'),
        COALESCE(pd.color, 'not specified'),
        CASE WHEN pd.is_vintage         THEN E'Vintage. '         ELSE '' END,
        CASE WHEN pd.is_limited_edition THEN E'Limited edition. ' ELSE '' END,
        CASE WHEN pd.is_collab          THEN 'Collab: ' || pd.collab_name || '. ' ELSE '' END,
        COALESCE(pd.special_notes, ''),
        CASE WHEN p.ai_notes IS NOT NULL THEN E'\n' || p.ai_notes ELSE '' END
    )
    INTO result
    FROM product p
    JOIN brand     b  ON p.brand_id    = b.id
    JOIN category  c  ON p.category_id = c.id
    LEFT JOIN category        cp ON c.parent_id  = cp.id
    LEFT JOIN product_details pd ON p.id = pd.product_id
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(st.name, ', ' ORDER BY st.name) AS tags
        FROM product_style_tag pst JOIN style_tag st ON pst.style_tag_id = st.id
        WHERE pst.product_id = p.id
    ) styles ON true
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(vt.name, ', ' ORDER BY vt.name) AS tags
        FROM product_vibe_tag pvt JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
        WHERE pvt.product_id = p.id
    ) vibes ON true
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(s.name, ', ' ORDER BY s.name) AS tags
        FROM product_season psn JOIN season s ON psn.season_id = s.id
        WHERE psn.product_id = p.id
    ) seasons ON true
    WHERE p.id = p_product_id;

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION generate_product_text IS 'Builds text representation for the Indexing Service (embedding generation)';

COMMIT;