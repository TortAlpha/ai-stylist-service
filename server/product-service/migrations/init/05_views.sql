-- ============================================================
-- VIEW: FULL PRODUCT CARD
-- ============================================================

CREATE VIEW v_product_full AS
SELECT
    p.id,
    p.sku,
    p.name,
    p.purchase_price,
    pl.name AS purchase_location,
    p.currency,
    p.ai_notes,
    p.category_id,
    p.image_count,
    p.version,

    p.status,
    c.product_type AS type,

    -- Brand
    b.id   AS brand_id,
    b.name AS brand_name,
    b.tier AS brand_tier,

    -- Category
    c.name   AS category_name,
    cp.name  AS parent_category,
    c.gender,

    -- Details
    pd.material,
    pd.condition,
    pd.color,
    pd.year_of_release,
    pd.is_vintage,
    pd.is_collab,
    pd.collab_name,
    pd.is_limited_edition,
    pd.special_notes,

    -- Sizing (unified)
    c.size_group,
    pd.size_value,
    pd.size_value2,
    pd.size_system,
    pd.measurement_cm,

    -- Clothing
    pd.fit           AS clothing_fit,

    -- Footwear
    pd.shoe_width,
    pd.insole_length_cm,

    -- Bags
    pd.width_cm      AS bag_width_cm,
    pd.height_cm     AS bag_height_cm,
    pd.depth_cm      AS bag_depth_cm,
    pd.handle_type   AS bag_handle_type,
    pd.bag_size_label,

    -- Jewelry
    pd.metal         AS jewelry_metal,
    pd.stone         AS jewelry_stone,
    pd.clasp_type    AS jewelry_clasp_type,

    -- Tags
    COALESCE(styles.tags,  '{}') AS style_tags,
    COALESCE(vibes.tags,   '{}') AS vibe_tags,
    COALESCE(seasons.tags, '{}') AS season_tags,

    p.created_at,
    p.updated_at

FROM product p
JOIN brand     b  ON p.brand_id    = b.id
JOIN category  c  ON p.category_id = c.id
LEFT JOIN purchase_location pl ON p.purchase_location_id = pl.id
LEFT JOIN category           cp ON c.parent_id   = cp.id
LEFT JOIN product_details    pd ON p.id = pd.product_id

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(st.name ORDER BY st.name) AS tags
    FROM product_style_tag pst
    JOIN style_tag st ON pst.style_tag_id = st.id
    WHERE pst.product_id = p.id
) styles ON true

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(vt.name ORDER BY vt.name) AS tags
    FROM product_vibe_tag pvt
    JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
    WHERE pvt.product_id = p.id
) vibes ON true

LEFT JOIN LATERAL (
    SELECT ARRAY_AGG(s.name ORDER BY s.name) AS tags
    FROM product_season psn
    JOIN season s ON psn.season_id = s.id
    WHERE psn.product_id = p.id
) seasons ON true

WHERE p.is_deleted = false;

COMMENT ON VIEW v_product_full IS 'Full product card with all details and tags';

-- ============================================================
-- VIEW: ANALYTICS EXPORT
-- ============================================================

CREATE VIEW v_product_analytics_export AS
SELECT
    p.id AS product_id,
    p.sku,
    p.name,
    p.version AS product_version,
    p.status,
    p.purchase_price,
    p.currency,
    p.purchase_location_id,
    pl.name AS purchase_location,
    p.image_count,

    b.id AS brand_id,
    b.name AS brand_name,
    b.tier AS brand_tier,
    b.country AS brand_country,

    c.id AS category_id,
    c.name AS category_name,
    cp.id AS parent_category_id,
    cp.name AS parent_category,
    c.gender,
    c.product_type,
    c.size_group,

    pd.material,
    pd.condition,
    pd.color,
    pd.year_of_release,
    pd.is_vintage,
    pd.is_collab,
    pd.collab_name,
    pd.is_limited_edition,
    pd.size_value,
    pd.size_value2,
    pd.size_system,
    pd.measurement_cm,
    pd.fit,
    pd.shoe_width,
    pd.insole_length_cm,
    pd.width_cm,
    pd.height_cm,
    pd.depth_cm,
    pd.handle_type,
    pd.bag_size_label,
    pd.metal,
    pd.stone,
    pd.clasp_type,

    COALESCE(styles.tag_ids, ARRAY[]::INT[]) AS style_tag_ids,
    COALESCE(styles.tags, ARRAY[]::TEXT[]) AS style_tags,
    COALESCE(vibes.tag_ids, ARRAY[]::INT[]) AS vibe_tag_ids,
    COALESCE(vibes.tags, ARRAY[]::TEXT[]) AS vibe_tags,
    COALESCE(seasons.tag_ids, ARRAY[]::INT[]) AS season_ids,
    COALESCE(seasons.tags, ARRAY[]::TEXT[]) AS season_tags,

    p.is_deleted,
    p.deleted_at,
    p.created_at,
    p.updated_at

FROM product p
JOIN brand b ON p.brand_id = b.id
JOIN category c ON p.category_id = c.id
LEFT JOIN purchase_location pl ON p.purchase_location_id = pl.id
LEFT JOIN category cp ON c.parent_id = cp.id
LEFT JOIN product_details pd ON p.id = pd.product_id

LEFT JOIN LATERAL (
    SELECT
        ARRAY_AGG(st.id ORDER BY st.name, st.id) AS tag_ids,
        ARRAY_AGG(st.name ORDER BY st.name, st.id) AS tags
    FROM product_style_tag pst
    JOIN style_tag st ON pst.style_tag_id = st.id
    WHERE pst.product_id = p.id
) styles ON true

LEFT JOIN LATERAL (
    SELECT
        ARRAY_AGG(vt.id ORDER BY vt.name, vt.id) AS tag_ids,
        ARRAY_AGG(vt.name ORDER BY vt.name, vt.id) AS tags
    FROM product_vibe_tag pvt
    JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
    WHERE pvt.product_id = p.id
) vibes ON true

LEFT JOIN LATERAL (
    SELECT
        ARRAY_AGG(s.id ORDER BY s.name, s.id) AS tag_ids,
        ARRAY_AGG(s.name ORDER BY s.name, s.id) AS tags
    FROM product_season psn
    JOIN season s ON psn.season_id = s.id
    WHERE psn.product_id = p.id
) seasons ON true;

COMMENT ON VIEW v_product_analytics_export IS 'Flattened product snapshot for analytics-service bootstrap, backfill, and reconciliation';

-- ============================================================
-- VIEW: STOREFRONT (chat bot & catalog)
-- ============================================================

CREATE VIEW v_product_storefront AS
SELECT * FROM v_product_full
WHERE status = 'ready';

COMMENT ON VIEW v_product_storefront IS 'Products visible to customers — only items with status ready';
