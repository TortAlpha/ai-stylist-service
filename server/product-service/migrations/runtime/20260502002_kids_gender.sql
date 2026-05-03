-- Add 'kids' as a new value for category.gender, plus the kids category subtree.
--
-- Changes:
--   1. Recreate category.gender CHECK constraint to include 'kids'.
--   2. Update generate_product_sku() to map 'kids' -> 'K' in the SKU.
--   3. Insert 7 root kids categories (Tops, Bottoms, Outerwear,
--      One-Piece & Sets, Footwear, Accessories, Bags). Jewelry intentionally
--      skipped at this stage.
--   4. Insert kids subcategories under each root.
--
-- Idempotent:
--   - Constraint is dropped IF EXISTS before re-add.
--   - Function is CREATE OR REPLACE.
--   - All category INSERTs use ON CONFLICT DO NOTHING (covered by
--     UNIQUE(name, parent_id, gender) and the partial unique index for roots).

BEGIN;

-- 1. Recreate gender CHECK to include 'kids'
ALTER TABLE category DROP CONSTRAINT IF EXISTS category_gender_check;
ALTER TABLE category ADD CONSTRAINT category_gender_check
    CHECK (gender IN ('male', 'female', 'unisex', 'kids'));

-- 2. Update SKU generator: add 'kids' -> 'K' mapping
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
        WHEN 'kids'   THEN 'K'
    END
    INTO v_category_code, v_gender_code
    FROM category WHERE id = NEW.category_id;

    v_seq := nextval('product_sku_seq');

    NEW.sku = 'AVA-' || v_brand_code || '-' || v_gender_code || '-' || v_category_code || '-' || LPAD(v_seq::TEXT, 5, '0');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 3. Kids root categories (7)
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
('Tops',             'TOP', NULL, 'kids', 'clothing',    'letter'),
('Bottoms',          'BTM', NULL, 'kids', 'clothing',    'letter'),
('Outerwear',        'OUT', NULL, 'kids', 'clothing',    'letter_or_numeric'),
('One-Piece & Sets', 'SET', NULL, 'kids', 'clothing',    'letter'),
('Footwear',         'FTW', NULL, 'kids', 'footwear',    'shoe'),
('Accessories',      'ACC', NULL, 'kids', 'accessories', 'one_size'),
('Bags',             'BAG', NULL, 'kids', 'bags',        'dimensions')
ON CONFLICT DO NOTHING;

-- 4. Kids subcategories
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
-- Kids Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
-- Kids Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'waist_length'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'waist_length'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Leggings',              'LEG', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
-- Kids Outerwear
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
('Coats',                 'COT', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
('Snowsuits & Overalls',  'SNO', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
-- Kids One-Piece & Sets
('Bodysuits & Rompers',   'BDS', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Jumpsuits',             'JMP', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
('Dresses',               'DRS', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter_or_numeric'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'clothing', 'letter'),
-- Kids Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'footwear', 'shoe'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'footwear', 'shoe'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'footwear', 'shoe'),
('Rain Boots',            'RNB', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'footwear', 'shoe'),
-- Kids Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'accessories', 'hat'),
('Scarves & Mittens',     'SCM', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'accessories', 'one_size'),
('Socks & Tights',        'SCK', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'accessories', 'one_size'),
-- Kids Bags
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'kids'), 'kids', 'bags', 'dimensions')
ON CONFLICT DO NOTHING;

COMMIT;
