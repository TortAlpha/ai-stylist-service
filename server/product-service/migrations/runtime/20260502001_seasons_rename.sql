-- Rename seasons:
--   demi-season -> spring (весна)
--   all-season  -> autumn (осень)
--
-- Rationale: dropped the abstract "demi/all" buckets in favour of the four
-- concrete seasons, without re-allocating ids or breaking existing FKs.
--
-- Idempotent: re-running is a no-op (UPDATEs match nothing on the second run,
-- INSERTs are guarded by ON CONFLICT).

-- Rename only when the target name is not already present. season.name is UNIQUE,
-- so a naked UPDATE would crash if both legacy ('all-season'/'demi-season') and
-- canonical ('autumn'/'spring') rows coexist (e.g. after a manual intervention).
-- In that edge case both rows are left untouched here — the operator can re-link
-- product associations and drop the legacy row separately.
UPDATE season SET name = 'autumn' WHERE name = 'all-season'
  AND NOT EXISTS (SELECT 1 FROM season WHERE name = 'autumn');
UPDATE season SET name = 'spring' WHERE name = 'demi-season'
  AND NOT EXISTS (SELECT 1 FROM season WHERE name = 'spring');

INSERT INTO season (name) VALUES
    ('spring'), ('autumn')
ON CONFLICT (name) DO NOTHING;
