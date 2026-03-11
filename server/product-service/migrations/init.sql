-- ============================================================
-- Product Service — PostgreSQL Init Script
-- Branded second-hand clothing store
--
-- Includes:
--   - Lookup tables (brand, category, tags, seasons, statuses)
--   - Product with versioning, statuses, reservations
--   - Product details and junction tables
--   - Product history (audit log)
--   - Full product card view
--   - Storefront view (chat bot & catalog)
--   - Text generation helper for Indexing Service
-- ============================================================

BEGIN;

-- ============================================================
-- EXTENSIONS
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";   -- UUID generation
CREATE EXTENSION IF NOT EXISTS "pg_trgm";     -- fuzzy text search

-- ============================================================
-- LOOKUP TABLES
-- ============================================================

-- ------- Brands -------

CREATE TABLE brand (
    id          SERIAL PRIMARY KEY,
    name        VARCHAR(200) NOT NULL UNIQUE,
    tier        VARCHAR(20)  NOT NULL CHECK (tier IN ('mass', 'premium', 'luxury')),
    country     VARCHAR(100),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE  brand      IS 'Brand directory';
COMMENT ON COLUMN brand.tier IS 'Market segment: mass-market, premium, luxury';

-- ------- Categories (hierarchical) -------

CREATE TABLE category (
    id               SERIAL PRIMARY KEY,
    name             VARCHAR(200) NOT NULL,
    parent_category  VARCHAR(200),
    gender           VARCHAR(10)  NOT NULL CHECK (gender IN ('male', 'female', 'unisex')),
    UNIQUE (name, gender)
);

COMMENT ON TABLE  category                 IS 'Hierarchical product categories';
COMMENT ON COLUMN category.parent_category IS 'Parent category (e.g. footwear -> sneakers)';

-- ------- Style tags -------

CREATE TABLE style_tag (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

COMMENT ON TABLE style_tag IS 'Style classification tags (streetwear, formal, casual, etc.)';

-- ------- Vibe tags -------

CREATE TABLE vibe_tag (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

COMMENT ON TABLE vibe_tag IS 'Mood / vibe tags (hype, minimalism, retro, etc.)';

-- ------- Seasons -------

CREATE TABLE season (
    id   SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

COMMENT ON TABLE season IS 'Season applicability for products';

INSERT INTO season (name) VALUES
    ('summer'), ('winter'), ('demi-season'), ('all-season');

-- ------- Product statuses (lifecycle) -------

CREATE TABLE product_status (
    id          SERIAL PRIMARY KEY,
    code        VARCHAR(50)  NOT NULL UNIQUE,
    name        VARCHAR(200) NOT NULL,
    description TEXT,
    sort_order  INT NOT NULL DEFAULT 0
);

COMMENT ON TABLE  product_status            IS 'Product lifecycle statuses — defines the workflow from intake to sold';
COMMENT ON COLUMN product_status.code       IS 'Machine-readable status identifier';
COMMENT ON COLUMN product_status.sort_order IS 'Workflow sequence number for UI ordering';

INSERT INTO product_status (code, name, description, sort_order) VALUES
    ('intake',       'Intake',           'Item received, pending inspection',            10),
    ('inspection',   'Inspection',       'Authenticity and condition check in progress',  20),
    ('rejected',     'Rejected',         'Failed inspection — counterfeit or damaged',    25),
    ('preparation',  'Preparation',      'Cleaning, steaming, minor repairs',             30),
    ('photo_queue',  'Photo Queue',      'Waiting for product photography',               40),
    ('photo_done',   'Photo Done',       'Photos taken, pending description',              50),
    ('ready',        'Ready for Sale',   'Listed and available for purchase',              60),
    ('reserved',     'Reserved',         'Held for a customer (temporary)',                70),
    ('sold',         'Sold',             'Purchase completed',                             80),
    ('returned',     'Returned',         'Customer returned the item',                     90);

-- ============================================================
-- MAIN PRODUCT TABLE
-- ============================================================

CREATE TABLE product (
    id                UUID           PRIMARY KEY DEFAULT uuid_generate_v4(),
    name              VARCHAR(500)   NOT NULL,
    brand_id          INT            NOT NULL REFERENCES brand(id),
    category_id       INT            NOT NULL REFERENCES category(id),
    status_id         INT            NOT NULL DEFAULT 1 REFERENCES product_status(id),
    price             DECIMAL(12, 2) NOT NULL CHECK (price > 0),
    currency          VARCHAR(3)     NOT NULL DEFAULT 'EUR',
    in_stock          BOOLEAN        NOT NULL DEFAULT true,
    quantity          INT            NOT NULL DEFAULT 1 CHECK (quantity >= 0),
    images_path       TEXT,
    preview_image_url TEXT,
    product_url       TEXT,
    version           INT            NOT NULL DEFAULT 1,
    reserved_until    TIMESTAMPTZ,
    reserved_by       VARCHAR(200),
    created_at        TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_brand    ON product (brand_id);
CREATE INDEX idx_product_category ON product (category_id);
CREATE INDEX idx_product_status   ON product (status_id);
CREATE INDEX idx_product_in_stock ON product (in_stock) WHERE in_stock = true;
CREATE INDEX idx_product_price    ON product (price);

COMMENT ON TABLE  product                       IS 'Core product table — one row per item in the store';
COMMENT ON COLUMN product.images_path           IS 'Path to image directory in storage (e.g. S3 bucket prefix)';
COMMENT ON COLUMN product.preview_image_url     IS 'Main product thumbnail URL — used in chat responses and catalog cards';
COMMENT ON COLUMN product.product_url           IS 'Link to the full product page on the storefront';
COMMENT ON COLUMN product.version               IS 'Optimistic lock counter — incremented on every update, used by Indexing Service to detect changes';
COMMENT ON COLUMN product.status_id             IS 'Current lifecycle status — controls visibility and available actions';
COMMENT ON COLUMN product.reserved_until        IS 'Reservation expiry — auto-release back to ready after this time';
COMMENT ON COLUMN product.reserved_by           IS 'Customer identifier who placed the reservation';

-- ============================================================
-- PRODUCT DETAILS
-- ============================================================

CREATE TABLE product_details (
    product_id         UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    material           VARCHAR(200),
    condition          VARCHAR(30) NOT NULL CHECK (
                           condition IN ('new_with_tags', 'excellent', 'good', 'fair')
                       ),
    color              VARCHAR(100),
    size               VARCHAR(30),
    fit                VARCHAR(20) CHECK (fit IN ('regular', 'slim', 'oversized', 'relaxed')),
    year_of_release    INT CHECK (year_of_release BETWEEN 1900 AND 2100),
    is_vintage         BOOLEAN NOT NULL DEFAULT false,
    is_collab          BOOLEAN NOT NULL DEFAULT false,
    collab_name        VARCHAR(300),
    is_limited_edition BOOLEAN NOT NULL DEFAULT false,
    special_notes      TEXT
);

CREATE INDEX idx_details_condition ON product_details (condition);
CREATE INDEX idx_details_size      ON product_details (size);

COMMENT ON TABLE  product_details              IS 'Extended product attributes (material, condition, fit, etc.)';
COMMENT ON COLUMN product_details.condition    IS 'Item condition: new_with_tags, excellent, good, fair';
COMMENT ON COLUMN product_details.collab_name  IS 'Collaboration name if applicable (e.g. Nike x Off-White)';

-- ============================================================
-- JUNCTION TABLES (many-to-many relationships)
-- ============================================================

-- Product <-> Style tags
CREATE TABLE product_style_tag (
    product_id   UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    style_tag_id INT  NOT NULL REFERENCES style_tag(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, style_tag_id)
);

COMMENT ON TABLE product_style_tag IS 'Junction: product <-> style tags (many-to-many)';

-- Product <-> Vibe tags
CREATE TABLE product_vibe_tag (
    product_id  UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    vibe_tag_id INT  NOT NULL REFERENCES vibe_tag(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, vibe_tag_id)
);

COMMENT ON TABLE product_vibe_tag IS 'Junction: product <-> vibe tags (many-to-many)';

-- Product <-> Seasons
CREATE TABLE product_season (
    product_id UUID NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    season_id  INT  NOT NULL REFERENCES season(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, season_id)
);

COMMENT ON TABLE product_season IS 'Junction: product <-> seasons (many-to-many)';

-- ============================================================
-- SIMILAR PRODUCTS
-- ============================================================

CREATE TABLE similar_products (
    product_id         UUID  NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    similar_product_id UUID  NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    similarity_score   FLOAT NOT NULL CHECK (similarity_score BETWEEN 0 AND 1),
    PRIMARY KEY (product_id, similar_product_id),
    CHECK (product_id != similar_product_id)
);

COMMENT ON TABLE  similar_products                  IS 'Pre-computed product similarity pairs for recommendations';
COMMENT ON COLUMN similar_products.similarity_score IS 'Cosine similarity between product embeddings (0..1)';

-- ============================================================
-- PRODUCT HISTORY (audit log)
-- ============================================================

CREATE TABLE product_history (
    id                BIGSERIAL PRIMARY KEY,
    product_id        UUID           NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    version           INT            NOT NULL,
    name              VARCHAR(500),
    brand_id          INT,
    category_id       INT,
    status_id         INT,
    price             DECIMAL(12, 2),
    currency          VARCHAR(3),
    in_stock          BOOLEAN,
    quantity          INT,
    images_path       TEXT,
    preview_image_url TEXT,
    product_url       TEXT,
    reserved_until    TIMESTAMPTZ,
    reserved_by       VARCHAR(200),
    changed_by        VARCHAR(200),
    changed_at        TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_product_history_product ON product_history (product_id, version);

COMMENT ON TABLE  product_history            IS 'Audit log — snapshot of product state before each update';
COMMENT ON COLUMN product_history.changed_by IS 'Who made the change (admin username, system, cron, etc.)';

-- ============================================================
-- TRIGGERS
-- ============================================================

-- Auto-update updated_at on any row change
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_product_updated_at
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Auto-increment version + save snapshot to history
CREATE OR REPLACE FUNCTION track_product_version()
RETURNS TRIGGER AS $$
BEGIN
    -- Save old state to history
    INSERT INTO product_history (
        product_id, version, name, brand_id, category_id, status_id,
        price, currency, in_stock, quantity,
        images_path, preview_image_url, product_url,
        reserved_until, reserved_by,
        changed_at
    ) VALUES (
        OLD.id, OLD.version, OLD.name, OLD.brand_id, OLD.category_id, OLD.status_id,
        OLD.price, OLD.currency, OLD.in_stock, OLD.quantity,
        OLD.images_path, OLD.preview_image_url, OLD.product_url,
        OLD.reserved_until, OLD.reserved_by,
        now()
    );

    -- Increment version
    NEW.version = OLD.version + 1;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_product_version
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION track_product_version();

-- ============================================================
-- Auto-set status to 'sold' when quantity reaches 0
-- Auto-set in_stock to false
-- ============================================================

CREATE OR REPLACE FUNCTION handle_product_quantity()
RETURNS TRIGGER AS $$
DECLARE
    sold_status_id INT;
    ready_status_id INT;
BEGIN
    -- When quantity drops to 0 — mark as sold
    IF NEW.quantity = 0 AND OLD.quantity > 0 THEN
        SELECT id INTO sold_status_id
        FROM product_status WHERE code = 'sold';

        NEW.in_stock = false;
        NEW.status_id = sold_status_id;
    END IF;

    -- When quantity goes back up (e.g. return) — mark as ready
    IF NEW.quantity > 0 AND OLD.quantity = 0 THEN
        SELECT id INTO ready_status_id
        FROM product_status WHERE code = 'ready';

        NEW.in_stock = true;
        NEW.status_id = ready_status_id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_product_quantity
    BEFORE UPDATE ON product
    FOR EACH ROW
    WHEN (OLD.quantity IS DISTINCT FROM NEW.quantity)
    EXECUTE FUNCTION handle_product_quantity();

-- ============================================================
-- VIEW: FULL PRODUCT CARD
-- Aggregates all related data into a single row per product
-- ============================================================

CREATE VIEW v_product_full AS
SELECT
    p.id,
    p.name,
    p.price,
    p.currency,
    p.in_stock,
    p.quantity,
    p.preview_image_url,
    p.product_url,
    p.version,

    -- Status
    ps.code AS status_code,
    ps.name AS status_name,

    -- Reservation
    p.reserved_until,
    p.reserved_by,

    -- Brand info
    b.name AS brand_name,
    b.tier AS brand_tier,

    -- Category info
    c.name            AS category_name,
    c.parent_category AS parent_category,
    c.gender,

    -- Product details
    pd.material,
    pd.condition,
    pd.color,
    pd.size,
    pd.fit,
    pd.year_of_release,
    pd.is_vintage,
    pd.is_collab,
    pd.collab_name,
    pd.is_limited_edition,
    pd.special_notes,

    -- Aggregated tags as arrays
    COALESCE(styles.tags,  '{}') AS style_tags,
    COALESCE(vibes.tags,   '{}') AS vibe_tags,
    COALESCE(seasons.tags, '{}') AS season_tags,

    p.created_at,
    p.updated_at

FROM product p
JOIN product_status ps ON p.status_id   = ps.id
JOIN brand          b  ON p.brand_id    = b.id
JOIN category       c  ON p.category_id = c.id
LEFT JOIN product_details pd ON p.id = pd.product_id

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
) seasons ON true;

COMMENT ON VIEW v_product_full IS 'Full product card with all tags and status — used for API responses and embedding text generation';

-- ============================================================
-- VIEW: STOREFRONT (chat bot & catalog)
-- Pre-filtered to only show products available for sale
-- ============================================================

CREATE VIEW v_product_storefront AS
SELECT * FROM v_product_full
WHERE status_code = 'ready'
  AND in_stock = true;

COMMENT ON VIEW v_product_storefront IS 'Products visible to customers — only items with status ready and in stock';

-- ============================================================
-- HELPER FUNCTION
-- Generates a text block from all product fields
-- Called by Indexing Service to produce input for embedding model
-- ============================================================

CREATE OR REPLACE FUNCTION generate_product_text(p_product_id UUID)
RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    SELECT FORMAT(
        E'%s — %s %s, %s, size %s.\n'
        E'Brand: %s (%s). Category: %s > %s.\n'
        E'Style: %s. Vibe: %s. Season: %s.\n'
        E'Material: %s. Condition: %s. Fit: %s.\n'
        E'Color: %s. Price: %s %s.\n'
        E'%s%s%s'
        E'%s',
        -- Line 1: name and basic classification
        p.name,
        c.gender,
        c.parent_category,
        c.name,
        COALESCE(pd.size, 'n/a'),
        -- Line 2: brand and category path
        b.name,
        b.tier,
        COALESCE(c.parent_category, ''),
        c.name,
        -- Line 3: tags
        COALESCE(styles.tags, 'not specified'),
        COALESCE(vibes.tags, 'not specified'),
        COALESCE(seasons.tags, 'not specified'),
        -- Line 4: physical attributes
        COALESCE(pd.material, 'not specified'),
        COALESCE(pd.condition, 'not specified'),
        COALESCE(pd.fit, 'not specified'),
        -- Line 5: color and price
        COALESCE(pd.color, 'not specified'),
        p.price,
        p.currency,
        -- Line 6: optional highlights
        CASE WHEN pd.is_vintage         THEN E'Vintage. '         ELSE '' END,
        CASE WHEN pd.is_limited_edition THEN E'Limited edition. ' ELSE '' END,
        CASE WHEN pd.is_collab          THEN 'Collab: ' || pd.collab_name || '. ' ELSE '' END,
        COALESCE(pd.special_notes, '')
    )
    INTO result
    FROM product p
    JOIN brand    b  ON p.brand_id    = b.id
    JOIN category c  ON p.category_id = c.id
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

COMMENT ON FUNCTION generate_product_text IS 'Builds a complete text representation of a product for the Indexing Service (embedding generation)';

-- ============================================================
-- SAMPLE DATA
-- ============================================================

-- Brands
INSERT INTO brand (name, tier, country) VALUES
    ('Nike',           'premium', 'USA'),
    ('Adidas',         'premium', 'Germany'),
    ('Gucci',          'luxury',  'Italy'),
    ('Zara',           'mass',    'Spain'),
    ('The North Face', 'premium', 'USA');

-- Categories
INSERT INTO category (name, parent_category, gender) VALUES
    ('sneakers',      'footwear',  'male'),
    ('sneakers',      'footwear',  'female'),
    ('joggers',       'pants',     'unisex'),
    ('dress pants',   'pants',     'male'),
    ('puffer jacket', 'outerwear', 'unisex'),
    ('t-shirt',       'tops',      'unisex');

-- Style tags
INSERT INTO style_tag (name) VALUES
    ('streetwear'), ('casual'), ('smart casual'),
    ('formal'), ('athleisure'), ('sporty');

-- Vibe tags
INSERT INTO vibe_tag (name) VALUES
    ('hype'), ('minimalism'), ('retro'),
    ('wardrobe staple'), ('bold'), ('classic');

-- Sample product: Nike Air Max 97 (status = ready for sale)
INSERT INTO product (id, name, brand_id, category_id, status_id, price, currency)
VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'Nike Air Max 97 Silver Bullet',
    1,   -- Nike
    1,   -- sneakers / male
    7,   -- ready
    8500.00,
    'RSD'
);

INSERT INTO product_details (
    product_id, material, condition, color, size, fit,
    year_of_release, is_vintage, is_collab, is_limited_edition
) VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'synthetic, leather',
    'excellent',
    'silver',
    '43',
    'regular',
    2022,
    false, false, false
);

INSERT INTO product_style_tag (product_id, style_tag_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 1),  -- streetwear
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 2);  -- casual

INSERT INTO product_vibe_tag (product_id, vibe_tag_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 1),  -- hype
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 3);  -- retro

INSERT INTO product_season (product_id, season_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 1),  -- summer
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', 3);  -- demi-season

-- ============================================================
-- Verify setup
-- ============================================================

-- Full product card (all statuses)
-- SELECT * FROM v_product_full;

-- Storefront view (only ready + in stock)
-- SELECT * FROM v_product_storefront;

-- Generated text for embedding
-- SELECT generate_product_text('a1b2c3d4-e5f6-7890-abcd-ef1234567890');

COMMIT;