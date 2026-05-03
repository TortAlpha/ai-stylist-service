-- Extend style_tag and vibe_tag dictionaries.
--
-- Style tags lean toward "category / archetype", vibe tags toward
-- "mood / atmosphere". Overlaps between the two lists (e.g. cozy, comfort,
-- everyday, streetwear, boho, chic) are intentional — both axes can describe
-- the same notion from different angles.
--
-- Existing values (streetwear, casual, smart casual, formal, athleisure,
-- sporty in style_tag; hype, minimalism, retro, wardrobe staple, bold,
-- classic in vibe_tag) are kept as-is and de-duplicated by ON CONFLICT.

INSERT INTO style_tag (name) VALUES
    ('casual'),
    ('cozy'),
    ('winter'),
    ('minimalist'),
    ('classic'),
    ('everyday'),
    ('streetwear'),
    ('outdoor'),
    ('comfort'),
    ('scandinavian style'),
    ('neutral style'),
    ('soft girl'),
    ('y2k'),
    ('après-ski'),
    ('loungewear'),
    ('vintage'),
    ('retro'),
    ('boho'),
    ('art'),
    ('chic')
ON CONFLICT (name) DO NOTHING;

INSERT INTO vibe_tag (name) VALUES
    -- Cozy / warmth
    ('cozy'), ('warm'), ('soft'), ('relaxed'), ('comfort'),
    -- Minimalism / calm
    ('basic'), ('everyday'), ('vintage old money'), ('romantic'),
    -- City / street
    ('streetwear'), ('grunge'),
    -- Nature / boho
    ('natural'), ('boho'), ('organic'), ('countryside'),
    -- Elegance
    ('elegant'), ('chic'), ('evening'),
    -- Active
    ('travel'), ('activewear'),
    -- Dark aesthetic
    ('dark'), ('gothic'), ('rock'),
    -- Seasonal atmosphere (does NOT overlap the season table)
    ('snow day'), ('rainy day'), ('vacation'), ('beach'), ('holiday'), ('festival')
ON CONFLICT (name) DO NOTHING;
