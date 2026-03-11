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
    code        VARCHAR(10)  NOT NULL UNIQUE,
    tier        VARCHAR(20)  NOT NULL CHECK (tier IN ('mass', 'premium', 'luxury')),
    country     VARCHAR(100),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON COLUMN brand.code IS 'Short uppercase code for SKU generation (e.g. NK, AD, GU)';

COMMENT ON TABLE  brand      IS 'Brand directory';
COMMENT ON COLUMN brand.tier IS 'Market segment: mass-market, premium, luxury';

-- ------- Categories (hierarchical) -------

CREATE TABLE category (
    id        SERIAL PRIMARY KEY,
    name      VARCHAR(200) NOT NULL,
    code      VARCHAR(10)  NOT NULL,
    parent_id INT          REFERENCES category(id),
    gender    VARCHAR(10)  NOT NULL CHECK (gender IN ('male', 'female', 'unisex')),
    UNIQUE (name, parent_id, gender)
);

CREATE INDEX idx_category_parent ON category (parent_id);

COMMENT ON TABLE  category           IS 'Hierarchical product categories (self-referencing tree)';
COMMENT ON COLUMN category.code      IS 'Short uppercase code for SKU generation (e.g. SNK, JGR, PFJ)';
COMMENT ON COLUMN category.parent_id IS 'Parent category FK (NULL = root category, e.g. footwear; non-NULL = child, e.g. sneakers)';

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

-- ------- Product types -------

CREATE TABLE product_type (
    id   SERIAL PRIMARY KEY,
    code VARCHAR(20)  NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL UNIQUE
);

COMMENT ON TABLE product_type IS 'Product type — determines which type-specific detail table to use';

INSERT INTO product_type (code, name) VALUES
    ('clothing',    'Clothing'),
    ('footwear',    'Footwear'),
    ('bags',        'Bags'),
    ('jewelry',     'Jewelry'),
    ('accessories', 'Accessories');

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
    type_id           INT            NOT NULL REFERENCES product_type(id),
    status_id         INT            NOT NULL REFERENCES product_status(id),
    original_price    DECIMAL(12, 2) NOT NULL CHECK (original_price > 0),
    discount          DECIMAL(5, 2)  NOT NULL DEFAULT 0 CHECK (discount >= 0 AND discount <= 100),
    final_price       DECIMAL(12, 2) NOT NULL,
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
CREATE INDEX idx_product_type     ON product (type_id);
CREATE INDEX idx_product_status   ON product (status_id);
CREATE INDEX idx_product_in_stock ON product (in_stock) WHERE in_stock = true;
CREATE INDEX idx_product_orig_price  ON product (original_price);
CREATE INDEX idx_product_final_price ON product (final_price);

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
    year_of_release    INT CHECK (year_of_release BETWEEN 1900 AND 2100),
    is_vintage         BOOLEAN NOT NULL DEFAULT false,
    is_collab          BOOLEAN NOT NULL DEFAULT false,
    collab_name        VARCHAR(300),
    is_limited_edition BOOLEAN NOT NULL DEFAULT false,
    special_notes      TEXT
);

CREATE INDEX idx_details_condition ON product_details (condition);

COMMENT ON TABLE  product_details              IS 'Common product attributes shared across all product types';
COMMENT ON COLUMN product_details.condition    IS 'Item condition: new_with_tags, excellent, good, fair';
COMMENT ON COLUMN product_details.collab_name  IS 'Collaboration name if applicable (e.g. Nike x Off-White)';

-- ============================================================
-- TYPE-SPECIFIC DETAIL TABLES
-- ============================================================

-- ------- Clothing -------
CREATE TABLE clothing_details (
    product_id UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    size       VARCHAR(30),
    fit        VARCHAR(20) CHECK (fit IN ('regular', 'slim', 'oversized', 'relaxed'))
);

CREATE INDEX idx_clothing_size ON clothing_details (size);

COMMENT ON TABLE clothing_details IS 'Clothing-specific attributes (size, fit)';

-- ------- Footwear -------
CREATE TABLE footwear_details (
    product_id   UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    shoe_size    VARCHAR(10),
    size_system  VARCHAR(5) NOT NULL DEFAULT 'EU' CHECK (size_system IN ('EU', 'US', 'UK'))
);

CREATE INDEX idx_footwear_size ON footwear_details (shoe_size);

COMMENT ON TABLE footwear_details IS 'Footwear-specific attributes (shoe size, sizing system)';

-- ------- Bags -------
CREATE TABLE bag_details (
    product_id  UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    width_cm    DECIMAL(6, 1),
    height_cm   DECIMAL(6, 1),
    depth_cm    DECIMAL(6, 1),
    handle_type VARCHAR(50) CHECK (handle_type IN ('shoulder', 'crossbody', 'hand', 'backpack', 'tote'))
);

COMMENT ON TABLE bag_details IS 'Bag-specific attributes (dimensions, handle type)';

-- ------- Jewelry -------
CREATE TABLE jewelry_details (
    product_id UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    metal      VARCHAR(100),
    stone      VARCHAR(100),
    clasp_type VARCHAR(50)
);

COMMENT ON TABLE jewelry_details IS 'Jewelry-specific attributes (metal, stone, clasp)';

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
-- SHARED TRIGGER FUNCTIONS
-- ============================================================

-- Auto-update updated_at on any row change (reused by multiple tables)
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- MARKETPLACES & PRODUCT LISTINGS (admin-only)
-- ============================================================

CREATE TABLE marketplace (
    id        SERIAL PRIMARY KEY,
    name      VARCHAR(200) NOT NULL UNIQUE,
    code      VARCHAR(20)  NOT NULL UNIQUE,
    base_url  TEXT
);

COMMENT ON TABLE  marketplace          IS 'External marketplace directory (eBay, Etsy, Telegram, etc.)';
COMMENT ON COLUMN marketplace.code     IS 'Short code for internal use (e.g. EBAY, ETSY, TG, KP)';
COMMENT ON COLUMN marketplace.base_url IS 'Base URL of the marketplace (optional)';

INSERT INTO marketplace (name, code, base_url) VALUES
    ('Telegram',           'TG',   'https://t.me'),
    ('eBay',               'EBAY', 'https://www.ebay.com'),
    ('Etsy',               'ETSY', 'https://www.etsy.com'),
    ('KupujemProdajem',    'KP',   'https://www.kupujemprodajem.com');

CREATE TABLE product_listing (
    id              SERIAL PRIMARY KEY,
    product_id      UUID         NOT NULL REFERENCES product(id) ON DELETE CASCADE,
    marketplace_id  INT          NOT NULL REFERENCES marketplace(id) ON DELETE CASCADE,
    external_id     VARCHAR(500),
    external_url    TEXT,
    listing_price   DECIMAL(12, 2) CHECK (listing_price > 0),
    sold_price      DECIMAL(12, 2) CHECK (sold_price > 0),
    currency        VARCHAR(3),
    status          VARCHAR(20)  NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'paused', 'sold', 'removed')),
    listed_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (product_id, marketplace_id)
);

CREATE INDEX idx_listing_product     ON product_listing (product_id);
CREATE INDEX idx_listing_marketplace ON product_listing (marketplace_id);
CREATE INDEX idx_listing_status      ON product_listing (status);

COMMENT ON TABLE  product_listing              IS 'Tracks where a product is listed on external marketplaces (admin-only)';
COMMENT ON COLUMN product_listing.external_id  IS 'Product identifier on the external platform';
COMMENT ON COLUMN product_listing.external_url   IS 'Direct link to the listing on the external platform';
COMMENT ON COLUMN product_listing.listing_price  IS 'Price override for this marketplace — if NULL, product.final_price is used';
COMMENT ON COLUMN product_listing.sold_price     IS 'Actual sale price after negotiation — filled when status = sold';
COMMENT ON COLUMN product_listing.currency       IS 'Currency for the listing price — if NULL, product.currency is used';
COMMENT ON COLUMN product_listing.status         IS 'Listing status: active, paused, sold, removed — independent from product lifecycle status';
COMMENT ON COLUMN product_listing.listed_at    IS 'When the product was posted/listed on the marketplace';

-- Auto-update updated_at on product_listing changes
CREATE TRIGGER trg_listing_updated_at
    BEFORE UPDATE ON product_listing
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

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
    type_id           INT,
    status_id         INT,
    original_price    DECIMAL(12, 2),
    discount          DECIMAL(5, 2),
    final_price       DECIMAL(12, 2),
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

-- Auto-generate SKU on insert: {BRAND_CODE}-{GENDER}-{CATEGORY_CODE}-{SEQ}
-- Example: NK-M-SNK-00001
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

CREATE TRIGGER trg_product_sku
    BEFORE INSERT ON product
    FOR EACH ROW EXECUTE FUNCTION generate_product_sku();

-- Auto-compute final_price from original_price and discount
CREATE OR REPLACE FUNCTION compute_final_price()
RETURNS TRIGGER AS $$
BEGIN
    NEW.final_price = ROUND(NEW.original_price * (1 - NEW.discount / 100), 2);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_product_final_price
    BEFORE INSERT OR UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION compute_final_price();

CREATE TRIGGER trg_product_updated_at
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Auto-increment version + save snapshot to history
CREATE OR REPLACE FUNCTION track_product_version()
RETURNS TRIGGER AS $$
BEGIN
    -- Save old state to history
    INSERT INTO product_history (
        product_id, version, name, brand_id, category_id, type_id, status_id,
        original_price, discount, final_price, currency, in_stock, quantity,
        images_path, preview_image_url, product_url,
        reserved_until, reserved_by,
        changed_at
    ) VALUES (
        OLD.id, OLD.version, OLD.name, OLD.brand_id, OLD.category_id, OLD.type_id, OLD.status_id,
        OLD.original_price, OLD.discount, OLD.final_price, OLD.currency, OLD.in_stock, OLD.quantity,
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
    p.sku,
    p.name,
    p.original_price,
    p.discount,
    p.final_price,
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

    -- Product type
    pt.code AS type_code,
    pt.name AS type_name,

    -- Brand info
    b.id   AS brand_id,
    b.name AS brand_name,
    b.tier AS brand_tier,

    -- Category info
    c.name          AS category_name,
    cp.name         AS parent_category,
    c.gender,

    -- Product details (common)
    pd.material,
    pd.condition,
    pd.color,
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
JOIN product_type   pt ON p.type_id     = pt.id
JOIN brand          b  ON p.brand_id    = b.id
JOIN category       c  ON p.category_id = c.id
LEFT JOIN category  cp ON c.parent_id   = cp.id
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
        E'%s — %s %s, %s. Type: %s.\n'
        E'Brand: %s (%s). Category: %s > %s.\n'
        E'Style: %s. Vibe: %s. Season: %s.\n'
        E'Material: %s. Condition: %s. Color: %s.\n'
        E'Price: %s %s (discount: %s%%).\n'
        E'%s%s%s'
        E'%s',
        -- Line 1: name and basic classification
        p.name,
        c.gender,
        COALESCE(cp.name, ''),
        c.name,
        pt.name,
        -- Line 2: brand and category path
        b.name,
        b.tier,
        COALESCE(cp.name, ''),
        c.name,
        -- Line 3: tags
        COALESCE(styles.tags, 'not specified'),
        COALESCE(vibes.tags, 'not specified'),
        COALESCE(seasons.tags, 'not specified'),
        -- Line 4: physical attributes
        COALESCE(pd.material, 'not specified'),
        COALESCE(pd.condition, 'not specified'),
        COALESCE(pd.color, 'not specified'),
        -- Line 5: price
        p.original_price,
        p.currency,
        p.discount,
        -- Line 6: optional highlights
        CASE WHEN pd.is_vintage         THEN E'Vintage. '         ELSE '' END,
        CASE WHEN pd.is_limited_edition THEN E'Limited edition. ' ELSE '' END,
        CASE WHEN pd.is_collab          THEN 'Collab: ' || pd.collab_name || '. ' ELSE '' END,
        COALESCE(pd.special_notes, '')
    )
    INTO result
    FROM product p
    JOIN product_type pt ON p.type_id     = pt.id
    JOIN brand        b  ON p.brand_id    = b.id
    JOIN category     c  ON p.category_id = c.id
    LEFT JOIN category cp ON c.parent_id   = cp.id
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
INSERT INTO brand (name, code, tier, country) VALUES
    ('Nike',           'NK',  'premium', 'USA'),
    ('Adidas',         'AD',  'premium', 'Germany'),
    ('Gucci',          'GU',  'luxury',  'Italy'),
    ('Zara',           'ZR',  'mass',    'Spain'),
    ('The North Face', 'TNF', 'premium', 'USA');

-- Categories (root)
INSERT INTO category (name, code, parent_id, gender) VALUES
    ('footwear',  'FTW', NULL, 'unisex'),
    ('pants',     'PNT', NULL, 'unisex'),
    ('outerwear', 'OTW', NULL, 'unisex'),
    ('tops',      'TOP', NULL, 'unisex');

-- Categories (children)
INSERT INTO category (name, code, parent_id, gender) VALUES
    ('sneakers',      'SNK', (SELECT id FROM category WHERE code = 'FTW' AND parent_id IS NULL), 'male'),
    ('sneakers',      'SNK', (SELECT id FROM category WHERE code = 'FTW' AND parent_id IS NULL), 'female'),
    ('joggers',       'JGR', (SELECT id FROM category WHERE code = 'PNT' AND parent_id IS NULL), 'unisex'),
    ('dress pants',   'DRP', (SELECT id FROM category WHERE code = 'PNT' AND parent_id IS NULL), 'male'),
    ('puffer jacket', 'PFJ', (SELECT id FROM category WHERE code = 'OTW' AND parent_id IS NULL), 'unisex'),
    ('t-shirt',       'TSH', (SELECT id FROM category WHERE code = 'TOP' AND parent_id IS NULL), 'unisex');

-- Style tags
INSERT INTO style_tag (name) VALUES
    ('streetwear'), ('casual'), ('smart casual'),
    ('formal'), ('athleisure'), ('sporty');

-- Vibe tags
INSERT INTO vibe_tag (name) VALUES
    ('hype'), ('minimalism'), ('retro'),
    ('wardrobe staple'), ('bold'), ('classic');

-- Sample product: Nike Air Max 97 (type = footwear, status = ready for sale)
-- SKU is auto-generated by trigger: AVA-NK-M-SNK-00001
INSERT INTO product (id, name, brand_id, category_id, type_id, status_id, original_price, discount, currency)
VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'Nike Air Max 97 Silver Bullet',
    (SELECT id FROM brand WHERE code = 'NK'),
    (SELECT id FROM category WHERE code = 'SNK' AND gender = 'male'),
    (SELECT id FROM product_type WHERE code = 'footwear'),
    (SELECT id FROM product_status WHERE code = 'ready'),
    8500.00,
    10.00, -- 10% discount
    'RSD'
);

INSERT INTO product_details (
    product_id, material, condition, color,
    year_of_release, is_vintage, is_collab, is_limited_edition
) VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'synthetic, leather',
    'excellent',
    'silver',
    2022,
    false, false, false
);

INSERT INTO footwear_details (product_id, shoe_size, size_system)
VALUES ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', '43', 'EU');

INSERT INTO product_style_tag (product_id, style_tag_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM style_tag WHERE name = 'streetwear')),
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM style_tag WHERE name = 'casual'));

INSERT INTO product_vibe_tag (product_id, vibe_tag_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM vibe_tag WHERE name = 'hype')),
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM vibe_tag WHERE name = 'retro'));

INSERT INTO product_season (product_id, season_id) VALUES
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM season WHERE name = 'summer')),
    ('a1b2c3d4-e5f6-7890-abcd-ef1234567890', (SELECT id FROM season WHERE name = 'demi-season'));

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