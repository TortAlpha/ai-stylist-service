-- 1. Создаем корневые категории для каждого пола
INSERT INTO category (name, code, parent_id, gender) VALUES
('Tops',              'TOP', NULL, 'female'),
('Outerwear & Suits', 'OUT', NULL, 'female'),
('Bottoms',           'BTM', NULL, 'female'),
('One-Piece & Sets',  'SET', NULL, 'female'),
('Footwear',          'FTW', NULL, 'female'),
('Accessories',       'ACC', NULL, 'female'),

('Tops',              'TOP', NULL, 'male'),
('Outerwear & Suits', 'OUT', NULL, 'male'),
('Bottoms',           'BTM', NULL, 'male'),
('One-Piece & Sets',  'SET', NULL, 'male'),
('Footwear',          'FTW', NULL, 'male'),
('Accessories',       'ACC', NULL, 'male'),

('Tops',              'TOP', NULL, 'unisex'),
('Outerwear',         'OUT', NULL, 'unisex'),
('Bottoms',           'BTM', NULL, 'unisex'),
('Footwear',          'FTW', NULL, 'unisex'),
('Accessories',       'ACC', NULL, 'unisex');


-- 2. ЖЕНСКИЕ КАТЕГОРИИ (Female)
INSERT INTO category (name, code, parent_id, gender) VALUES
-- Female Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Shirts & Blouses',      'SHB', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'female'), 'female'),
-- Female Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'female'), 'female'),
-- Female Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Skirts',                'SKT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Leggings',              'LEG', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'female'), 'female'),
-- Female One-Piece & Sets
('Dresses',               'DRS', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Jumpsuits & Rompers',   'JMP', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'female'), 'female'),
-- Female Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'female'), 'female'),
-- Female Accessories
('Bags & Backpacks',      'BAG', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'female'), 'female');


-- 3. МУЖСКИЕ КАТЕГОРИИ (Male)
INSERT INTO category (name, code, parent_id, gender) VALUES
-- Male Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Shirts',                'SHT', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Polo Shirts',           'POL', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Tank Tops',             'TNK', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Long Sleeves',          'LSV', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'male'), 'male'),
-- Male Outerwear & Suits
('Blazers & Suit Jackets','BLZ', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Suits',                 'SUT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Coats & Trench Coats',  'COT', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Vests',                 'VST', (SELECT id FROM category WHERE name = 'Outerwear & Suits' AND parent_id IS NULL AND gender = 'male'), 'male'),
-- Male Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Pants & Trousers',      'PNT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Kilts',                 'KLT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'male'), 'male'),
-- Male One-Piece & Sets
('Tracksuits',            'TRK', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Swimwear',              'SWM', (SELECT id FROM category WHERE name = 'One-Piece & Sets' AND parent_id IS NULL AND gender = 'male'), 'male'),
-- Male Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Shoes',                 'SHO', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Sandals & Slides',      'SND', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'male'), 'male'),
-- Male Accessories
('Bags & Backpacks',      'BAG', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Belts',                 'BLT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Ties',                  'TIE', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'male'), 'male');


-- 4. УНИСЕКС КАТЕГОРИИ (Unisex) - только базовые/streetwear вещи
INSERT INTO category (name, code, parent_id, gender) VALUES
-- Unisex Tops
('T-Shirts',              'TSH', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Hoodies & Sweatshirts', 'HOD', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Sweaters & Cardigans',  'SWC', (SELECT id FROM category WHERE name = 'Tops' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
-- Unisex Outerwear
('Jackets',               'JKT', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Windbreakers & Bombers','WND', (SELECT id FROM category WHERE name = 'Outerwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
-- Unisex Bottoms
('Jeans',                 'JNS', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Sweatpants',            'SWP', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Shorts',                'SRT', (SELECT id FROM category WHERE name = 'Bottoms' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
-- Unisex Footwear
('Sneakers',              'SNK', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Boots',                 'BOT', (SELECT id FROM category WHERE name = 'Footwear' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
-- Unisex Accessories
('Bags & Backpacks',      'BAG', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Hats & Caps',           'HAT', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Scarves',               'SCR', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex'),
('Sunglasses',            'SUN', (SELECT id FROM category WHERE name = 'Accessories' AND parent_id IS NULL AND gender = 'unisex'), 'unisex');