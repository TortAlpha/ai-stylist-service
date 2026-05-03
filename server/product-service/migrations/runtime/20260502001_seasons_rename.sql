-- Rename seasons:
--   demi-season -> spring (весна)
--   all-season  -> autumn (осень)
--
-- Rationale: dropped the abstract "demi/all" buckets in favour of the four
-- concrete seasons, without re-allocating ids or breaking existing FKs.
--
-- Idempotent: re-running is a no-op (UPDATEs match nothing on the second run,
-- INSERTs are guarded by ON CONFLICT).

UPDATE season SET name = 'autumn' WHERE name = 'all-season';
UPDATE season SET name = 'spring' WHERE name = 'demi-season';

INSERT INTO season (name) VALUES
    ('spring'), ('autumn')
ON CONFLICT (name) DO NOTHING;
