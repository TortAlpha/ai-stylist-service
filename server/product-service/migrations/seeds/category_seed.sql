-- ============================================================
-- Category Seed Script (v5 — with size_group)
-- ============================================================
-- product_type mapping:
--   clothing    → Tops, Outerwear & Suits, Bottoms, One-Piece & Sets
--   footwear    → Footwear
--   bags        → Bags
--   jewelry     → Jewelry
--   accessories → Accessories (hats, belts, scarves, sunglasses, ties)
--
-- size_group mapping:
--   letter           → t-shirts, polos, tank tops, long sleeves, hoodies, sweaters, sweatpants, shorts, leggings, tracksuits, swimwear, gloves
--   letter_or_numeric→ shirts, blouses, skirts, dresses, blazers, suits, jackets, coats, windbreakers, vests, jumpsuits
--   waist_length     → jeans, pants & trousers
--   shoe             → all footwear
--   ring             → rings
--   measurement_cm   → bracelets, necklaces & chains, belts
--   dimensions       → all bags
--   hat              → hats & caps
--   one_size         → earrings, brooches, scarves, sunglasses, ties
-- ============================================================

BEGIN;

-- 1. Корневые категории (size_group = default, overridden by leaf categories)
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
-- Female
('Tops',              'TOP', NULL, 'female', 'clothing',     'letter'),
('Outerwear & Suits', 'OUT', NULL, 'female', 'clothing',     'letter_or_numeric'),
('Bottoms',           'BTM', NULL, 'female', 'clothing',     'letter'),
('One-Piece & Sets',  'SET', NULL, 'female', 'clothing',     'letter'),
('Footwear',          'FTW', NULL, 'female', 'footwear',     'shoe'),
('Bags',              'BAG', NULL, 'female', 'bags',         'dimensions'),
('Jewelry',           'JWL', NULL, 'female', 'jewelry',      'one_size'),
('Accessories',       'ACC', NULL, 'female', 'accessories',  'one_size'),

-- Male
('Tops',              'TOP', NULL, 'male', 'clothing',       'letter'),
('Outerwear & Suits', 'OUT', NULL, 'male', 'clothing',       'letter_or_numeric'),
('Bottoms',           'BTM', NULL, 'male', 'clothing',       'letter'),
('One-Piece & Sets',  'SET', NULL, 'male', 'clothing',       'letter'),
('Footwear',          'FTW', NULL, 'male', 'footwear',       'shoe'),
('Bags',              'BAG', NULL, 'male', 'bags',           'dimensions'),
('Jewelry',           'JWL', NULL, 'male', 'jewelry',        'one_size'),
('Accessories',       'ACC', NULL, 'male', 'accessories',    'one_size'),

-- Unisex
('Tops',              'TOP', NULL, 'unisex', 'clothing',     'letter'),
('Outerwear',         'OUT', NULL, 'unisex', 'clothing',     'letter_or_numeric'),
('Bottoms',           'BTM', NULL, 'unisex', 'clothing',     'letter'),
('Footwear',          'FTW', NULL, 'unisex', 'footwear',     'shoe'),
('Bags',              'BAG', NULL, 'unisex', 'bags',         'dimensions'),
('Jewelry',           'JWL', NULL, 'unisex', 'jewelry',      'one_size'),
('Accessories',       'ACC', NULL, 'unisex', 'accessories',  'one_size')
ON CONFLICT DO NOTHING;


-- 2. ЖЕНСКИЕ КАТЕГОРИИ (Female)
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
-- Female Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Shirts & Blouses',      'SHB', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
-- Female Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
-- Female Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'waist_length'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'waist_length'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Skirts',                'SKT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Leggings',              'LEG', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
-- Female One-Piece & Sets
('Dresses',               'DRS', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Jumpsuits & Rompers',   'JMP', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter_or_numeric'),
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing', 'letter'),
-- Female Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear', 'shoe'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear', 'shoe'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear', 'shoe'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear', 'shoe'),
('Heels',                 'HEL', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear', 'shoe'),
-- Female Bags
('Shoulder Bags',         'SHB', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
('Tote Bags',             'TOT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
('Clutches',              'CLT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags', 'dimensions'),
-- Female Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry', 'measurement_cm'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry', 'measurement_cm'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry', 'ring'),
('Earrings',              'EAR', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry', 'one_size'),
('Brooches',              'BRO', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry', 'one_size'),
-- Female Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories', 'hat'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories', 'measurement_cm'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories', 'one_size'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories', 'one_size'),
('Gloves',                'GLV', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories', 'letter')
ON CONFLICT DO NOTHING;


-- 3. МУЖСКИЕ КАТЕГОРИИ (Male)
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
-- Male Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Shirts',                'SHT', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
-- Male Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter_or_numeric'),
-- Male Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'waist_length'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'waist_length'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
-- Male One-Piece & Sets
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing', 'letter'),
-- Male Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear', 'shoe'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear', 'shoe'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear', 'shoe'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear', 'shoe'),
-- Male Bags
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags', 'dimensions'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags', 'dimensions'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags', 'dimensions'),
('Tote Bags',             'TOT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags', 'dimensions'),
-- Male Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry', 'measurement_cm'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry', 'measurement_cm'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry', 'ring'),
-- Male Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'hat'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'measurement_cm'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'one_size'),
('Ties',                  'TIE', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'one_size'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'one_size'),
('Gloves',                'GLV', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories', 'letter')
ON CONFLICT DO NOTHING;


-- 4. УНИСЕКС КАТЕГОРИИ (Unisex)
INSERT INTO category (name, code, parent_id, gender, product_type, size_group) VALUES
-- Unisex Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter'),
-- Unisex Outerwear
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter_or_numeric'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter_or_numeric'),
-- Unisex Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'waist_length'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing', 'letter'),
-- Unisex Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear', 'shoe'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear', 'shoe'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear', 'shoe'),
-- Unisex Bags
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags', 'dimensions'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags', 'dimensions'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags', 'dimensions'),
-- Unisex Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry', 'measurement_cm'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry', 'measurement_cm'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry', 'ring'),
-- Unisex Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories', 'hat'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories', 'one_size'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories', 'one_size')
ON CONFLICT DO NOTHING;

COMMIT;
