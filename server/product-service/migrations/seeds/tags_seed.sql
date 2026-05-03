BEGIN;

INSERT INTO style_tag (name) VALUES
    ('streetwear'), ('casual'), ('smart casual'),
    ('formal'), ('athleisure'), ('sporty'),
    ('cozy'), ('winter'), ('minimalist'), ('classic'),
    ('everyday'), ('outdoor'), ('comfort'),
    ('scandinavian style'), ('neutral style'),
    ('soft girl'), ('y2k'), ('après-ski'), ('loungewear'),
    ('vintage'), ('retro'), ('boho'), ('art'), ('chic')
ON CONFLICT (name) DO NOTHING;

INSERT INTO vibe_tag (name) VALUES
    ('hype'), ('minimalism'), ('retro'),
    ('wardrobe staple'), ('bold'), ('classic'),
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
    ('snow day'), ('rainy day'), ('vacation'),
    ('beach'), ('holiday'), ('festival')
ON CONFLICT (name) DO NOTHING;

COMMIT;
