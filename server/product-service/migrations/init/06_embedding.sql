-- ============================================================
-- HELPER: Generate text for embedding model
-- ============================================================

CREATE OR REPLACE FUNCTION generate_product_text(p_product_id UUID)
RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    SELECT FORMAT(
        E'%s \u2014 %s %s, %s. Type: %s.\nBrand: %s (%s). Category: %s > %s.\nStyle: %s. Vibe: %s. Season: %s.\nMaterial: %s. Condition: %s. Color: %s.\n%s%s%s%s%s',
        p.name,
        c.gender,
        COALESCE(cp.name, ''),
        c.name,
        c.product_type,
        b.name,
        b.tier,
        COALESCE(cp.name, ''),
        c.name,
        COALESCE(styles.tags, 'not specified'),
        COALESCE(vibes.tags, 'not specified'),
        COALESCE(seasons.tags, 'not specified'),
        COALESCE(pd.material, 'not specified'),
        COALESCE(pd.condition, 'not specified'),
        COALESCE(pd.color, 'not specified'),
        CASE WHEN pd.is_vintage         THEN E'Vintage. '         ELSE '' END,
        CASE WHEN pd.is_limited_edition THEN E'Limited edition. ' ELSE '' END,
        CASE WHEN pd.is_collab          THEN 'Collab: ' || pd.collab_name || '. ' ELSE '' END,
        COALESCE(pd.special_notes, ''),
        CASE WHEN p.ai_notes IS NOT NULL THEN E'\n' || p.ai_notes ELSE '' END
    )
    INTO result
    FROM product p
    JOIN brand     b  ON p.brand_id    = b.id
    JOIN category  c  ON p.category_id = c.id
    LEFT JOIN category        cp ON c.parent_id  = cp.id
    LEFT JOIN product_details pd ON p.id = pd.product_id
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(st.name, ', ' ORDER BY st.name) AS tags
        FROM product_style_tag pst JOIN style_tag st ON pst.style_tag_id = st.id
        WHERE pst.product_id = p.id
    ) styles ON true
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(vt.name, ', ' ORDER BY vt.name) AS tags
        FROM product_vibe_tag pvt JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
        WHERE pvt.product_id = p.id
    ) vibes ON true
    LEFT JOIN LATERAL (
        SELECT STRING_AGG(s.name, ', ' ORDER BY s.name) AS tags
        FROM product_season psn JOIN season s ON psn.season_id = s.id
        WHERE psn.product_id = p.id
    ) seasons ON true
    WHERE p.id = p_product_id;

    RETURN result;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION generate_product_text IS 'Builds text representation for the Indexing Service (embedding generation)';
