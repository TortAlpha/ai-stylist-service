
BEGIN;

INSERT INTO style_tag (name) VALUES
    ('streetwear'), ('casual'), ('smart casual'),
    ('formal'), ('athleisure'), ('sporty')
ON CONFLICT (name) DO NOTHING;

INSERT INTO vibe_tag (name) VALUES
    ('hype'), ('minimalism'), ('retro'),
    ('wardrobe staple'), ('bold'), ('classic')
ON CONFLICT (name) DO NOTHING;

COMMIT;
