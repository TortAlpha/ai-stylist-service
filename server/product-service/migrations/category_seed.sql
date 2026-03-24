-- ============================================================
-- Category Seed Script
-- ============================================================
-- product_type mapping:
--   clothing    → Tops, Outerwear & Suits, Bottoms, One-Piece & Sets
--   footwear    → Footwear
--   bags        → Bags
--   jewelry     → Jewelry
--   accessories → Accessories (hats, belts, scarves, sunglasses, ties)
-- ============================================================

-- 1. Корневые категории
INSERT INTO category (name, code, parent_id, gender, product_type) VALUES
-- Female
('Tops',              'TOP', NULL, 'female', 'clothing'),
('Outerwear & Suits', 'OUT', NULL, 'female', 'clothing'),
('Bottoms',           'BTM', NULL, 'female', 'clothing'),
('One-Piece & Sets',  'SET', NULL, 'female', 'clothing'),
('Footwear',          'FTW', NULL, 'female', 'footwear'),
('Bags',              'BAG', NULL, 'female', 'bags'),
('Jewelry',           'JWL', NULL, 'female', 'jewelry'),
('Accessories',       'ACC', NULL, 'female', 'accessories'),

-- Male
('Tops',              'TOP', NULL, 'male', 'clothing'),
('Outerwear & Suits', 'OUT', NULL, 'male', 'clothing'),
('Bottoms',           'BTM', NULL, 'male', 'clothing'),
('One-Piece & Sets',  'SET', NULL, 'male', 'clothing'),
('Footwear',          'FTW', NULL, 'male', 'footwear'),
('Bags',              'BAG', NULL, 'male', 'bags'),
('Jewelry',           'JWL', NULL, 'male', 'jewelry'),
('Accessories',       'ACC', NULL, 'male', 'accessories'),

-- Unisex
('Tops',              'TOP', NULL, 'unisex', 'clothing'),
('Outerwear',         'OUT', NULL, 'unisex', 'clothing'),
('Bottoms',           'BTM', NULL, 'unisex', 'clothing'),
('Footwear',          'FTW', NULL, 'unisex', 'footwear'),
('Bags',              'BAG', NULL, 'unisex', 'bags'),
('Jewelry',           'JWL', NULL, 'unisex', 'jewelry'),
('Accessories',       'ACC', NULL, 'unisex', 'accessories');


-- 2. ЖЕНСКИЕ КАТЕГОРИИ (Female)
INSERT INTO category (name, code, parent_id, gender, product_type) VALUES
-- Female Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Shirts & Blouses',      'SHB', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
-- Female Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
-- Female Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Skirts',                'SKT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Leggings',              'LEG', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
-- Female One-Piece & Sets
('Dresses',               'DRS', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Jumpsuits & Rompers',   'JMP', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female', 'clothing'),
-- Female Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear'),
('Heels',                 'HEL', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female', 'footwear'),
-- Female Bags
('Shoulder Bags',         'SHB', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
('Tote Bags',             'TOT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
('Clutches',              'CLT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'female'), 'female', 'bags'),
-- Female Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry'),
('Earrings',              'EAR', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry'),
('Brooches',              'BRO', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'female'), 'female', 'jewelry'),
-- Female Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories'),
('Gloves',                'GLV', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female', 'accessories');


-- 3. МУЖСКИЕ КАТЕГОРИИ (Male)
INSERT INTO category (name, code, parent_id, gender, product_type) VALUES
-- Male Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Shirts',                'SHT', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
-- Male Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
-- Male Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
-- Male One-Piece & Sets
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male', 'clothing'),
-- Male Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male', 'footwear'),
-- Male Bags
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags'),
('Tote Bags',             'TOT', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'male'), 'male', 'bags'),
-- Male Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'male'), 'male', 'jewelry'),
-- Male Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories'),
('Ties',                  'TIE', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories'),
('Gloves',                'GLV', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male', 'accessories');


-- 4. УНИСЕКС КАТЕГОРИИ (Unisex)
INSERT INTO category (name, code, parent_id, gender, product_type) VALUES
-- Unisex Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
-- Unisex Outerwear
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
-- Unisex Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'clothing'),
-- Unisex Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'footwear'),
-- Unisex Bags
('Backpacks',             'BKP', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags'),
('Crossbody Bags',        'CRS', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags'),
('Waist Bags',            'WST', (SELECT id FROM category WHERE name = 'Bags' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'bags'),
-- Unisex Jewelry
('Necklaces & Chains',    'NCK', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry'),
('Bracelets',             'BRC', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry'),
('Rings',                 'RNG', (SELECT id FROM category WHERE name = 'Jewelry' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'jewelry'),
-- Unisex Accessories
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex', 'accessories');
