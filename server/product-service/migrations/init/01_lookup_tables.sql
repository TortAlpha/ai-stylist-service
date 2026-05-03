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
    gender       VARCHAR(10)  NOT NULL CHECK (gender IN ('male', 'female', 'unisex', 'kids')),
    product_type VARCHAR(20)  NOT NULL
                 CHECK (product_type IN ('clothing', 'footwear', 'bags', 'jewelry', 'accessories')),
    size_group   VARCHAR(20)  NOT NULL DEFAULT 'one_size'
                 CHECK (size_group IN (
                     'letter',            -- XS, S, M, L, XL, XXL (t-shirts, hoodies, sweatpants, swimwear...)
                     'letter_or_numeric', -- S/M/L OR 36/38/40 + size_system (shirts, dresses, blazers, coats...)
                     'waist_length',      -- waist W28 + optional length L32 (jeans, pants)
                     'shoe',              -- shoe_size + size_system + optional width (all footwear)
                     'ring',              -- ring_size + size_system (rings)
                     'measurement_cm',    -- length/circumference in cm (bracelets, necklaces, belts)
                     'dimensions',        -- w × h × d cm + optional brand label (bags)
                     'hat',               -- head circumference cm OR letter size (hats & caps)
                     'one_size'           -- no sizing (earrings, brooches, scarves, sunglasses, ties)
                 )),
    UNIQUE (name, parent_id, gender)
);

CREATE INDEX idx_category_parent       ON category (parent_id);
CREATE INDEX idx_category_product_type ON category (product_type);
CREATE INDEX idx_category_size_group   ON category (size_group);
CREATE UNIQUE INDEX uq_category_root_name_gender
    ON category (name, gender)
    WHERE parent_id IS NULL;

COMMENT ON TABLE  category                IS 'Hierarchical product categories (self-referencing tree)';
COMMENT ON COLUMN category.code           IS 'Short uppercase code for SKU generation (e.g. SNK, JGR, PFJ)';
COMMENT ON COLUMN category.parent_id      IS 'Parent category FK (NULL = root category)';
COMMENT ON COLUMN category.product_type   IS 'Product type — determines which detail fields are relevant (inherited from root category)';
COMMENT ON COLUMN category.size_group     IS 'Sizing group — determines which size fields are relevant for products in this category';

-- ------- Purchase locations -------

CREATE TABLE purchase_location (
    id         SERIAL PRIMARY KEY,
    name       VARCHAR(200) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT now()
);

COMMENT ON TABLE purchase_location IS 'Normalized sourcing locations/channels used for filtering and analytics';
COMMENT ON COLUMN purchase_location.name IS 'Where the item was sourced from (store, market, city, online, etc.)';

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
    ('summer'), ('winter'), ('spring'), ('autumn')
ON CONFLICT (name) DO NOTHING;
