-- ============================================================
-- TRIGGER FUNCTIONS
-- ============================================================

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-generate SKU: AVA-{BRAND}-{GENDER}-{CATEGORY}-{SEQ}
CREATE OR REPLACE FUNCTION generate_product_sku()
RETURNS TRIGGER AS $$
DECLARE
    v_brand_code    VARCHAR(10);
    v_category_code VARCHAR(10);
    v_gender_code   VARCHAR(1);
    v_seq           BIGINT;
BEGIN
    SELECT code INTO v_brand_code
    FROM brand WHERE id = NEW.brand_id;

    SELECT code, CASE gender
        WHEN 'male'   THEN 'M'
        WHEN 'female' THEN 'F'
        WHEN 'unisex' THEN 'U'
    END
    INTO v_category_code, v_gender_code
    FROM category WHERE id = NEW.category_id;

    v_seq := nextval('product_sku_seq');

    NEW.sku = 'AVA-' || v_brand_code || '-' || v_gender_code || '-' || v_category_code || '-' || LPAD(v_seq::TEXT, 5, '0');

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION current_audit_actor()
RETURNS VARCHAR(200) AS $$
BEGIN
    RETURN COALESCE(
        NULLIF(current_setting('app.changed_by', true), ''),
        NULLIF(current_setting('app.user_id', true), ''),
        current_user,
        'system'
    );
END;
$$ LANGUAGE plpgsql STABLE;

CREATE OR REPLACE FUNCTION append_unique_int(p_values INT[], p_value INT)
RETURNS INT[] AS $$
BEGIN
    IF p_values IS NULL THEN
        p_values := '{}';
    END IF;

    IF p_value = ANY(p_values) THEN
        RETURN p_values;
    END IF;

    RETURN array_append(p_values, p_value);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION jsonb_row_diff(p_old JSONB, p_new JSONB)
RETURNS JSONB AS $$
DECLARE
    v_result JSONB := '{}'::jsonb;
    v_key    TEXT;
    v_old    JSONB;
    v_new    JSONB;
BEGIN
    FOR v_key IN
        SELECT key
        FROM (
            SELECT jsonb_object_keys(COALESCE(p_old, '{}'::jsonb)) AS key
            UNION
            SELECT jsonb_object_keys(COALESCE(p_new, '{}'::jsonb)) AS key
        ) AS keys
    LOOP
        v_old := p_old -> v_key;
        v_new := p_new -> v_key;

        IF v_old IS DISTINCT FROM v_new THEN
            v_result := v_result || jsonb_build_object(
                v_key,
                jsonb_build_object('old', v_old, 'new', v_new)
            );
        END IF;
    END LOOP;

    RETURN v_result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION jsonb_snapshot_diff(p_snapshot JSONB, p_side TEXT)
RETURNS JSONB AS $$
DECLARE
    v_result JSONB := '{}'::jsonb;
    v_key    TEXT;
    v_value  JSONB;
BEGIN
    FOR v_key IN
        SELECT jsonb_object_keys(COALESCE(p_snapshot, '{}'::jsonb))
    LOOP
        v_value := p_snapshot -> v_key;

        IF v_value = 'null'::jsonb THEN
            CONTINUE;
        END IF;

        IF p_side = 'new' THEN
            v_result := v_result || jsonb_build_object(
                v_key,
                jsonb_build_object('old', NULL, 'new', v_value)
            );
        ELSIF p_side = 'old' THEN
            v_result := v_result || jsonb_build_object(
                v_key,
                jsonb_build_object('old', v_value, 'new', NULL)
            );
        ELSE
            RAISE EXCEPTION 'Unsupported snapshot diff side: %', p_side;
        END IF;
    END LOOP;

    RETURN v_result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION merge_audit_changes(p_existing JSONB, p_incoming JSONB)
RETURNS JSONB AS $$
DECLARE
    v_result   JSONB := COALESCE(p_existing, '{}'::jsonb);
    v_key      TEXT;
    v_existing JSONB;
    v_incoming JSONB;
    v_old      JSONB;
    v_new      JSONB;
BEGIN
    FOR v_key IN
        SELECT jsonb_object_keys(COALESCE(p_incoming, '{}'::jsonb))
    LOOP
        v_existing := v_result -> v_key;
        v_incoming := p_incoming -> v_key;

        IF v_existing IS NULL THEN
            v_result := v_result || jsonb_build_object(v_key, v_incoming);
            CONTINUE;
        END IF;

        v_old := v_existing -> 'old';
        v_new := v_incoming -> 'new';

        IF v_old IS NOT DISTINCT FROM v_new THEN
            v_result := v_result - v_key;
        ELSE
            v_result := v_result || jsonb_build_object(
                v_key,
                jsonb_build_object('old', v_old, 'new', v_new)
            );
        END IF;
    END LOOP;

    RETURN v_result;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

CREATE OR REPLACE FUNCTION queue_product_create()
RETURNS TRIGGER AS $$
DECLARE
    v_changes    JSONB;
    v_changed_by VARCHAR(200) := current_audit_actor();
    v_changed_at TIMESTAMPTZ := now();
BEGIN
    v_changes := jsonb_snapshot_diff(
        to_jsonb(NEW) - 'id' - 'version' - 'created_at' - 'updated_at',
        'new'
    );

    INSERT INTO product_audit_buffer (
        txid,
        product_id,
        created_in_tx,
        product_changes,
        changed_by,
        changed_at
    )
    VALUES (
        txid_current(),
        NEW.id,
        true,
        v_changes,
        v_changed_by,
        v_changed_at
    )
    ON CONFLICT (txid, product_id) DO UPDATE
    SET
        created_in_tx = true,
        product_changes = merge_audit_changes(product_audit_buffer.product_changes, EXCLUDED.product_changes),
        changed_by = EXCLUDED.changed_by,
        changed_at = EXCLUDED.changed_at;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_changes()
RETURNS TRIGGER AS $$
DECLARE
    v_changes    JSONB;
    v_changed_by VARCHAR(200) := current_audit_actor();
    v_changed_at TIMESTAMPTZ := now();
BEGIN
    v_changes := jsonb_row_diff(
        to_jsonb(OLD) - 'id' - 'sku' - 'version' - 'created_at' - 'updated_at',
        to_jsonb(NEW) - 'id' - 'sku' - 'version' - 'created_at' - 'updated_at'
    );

    IF v_changes = '{}'::jsonb THEN
        RETURN NEW;
    END IF;

    INSERT INTO product_audit_buffer (
        txid,
        product_id,
        product_changes,
        changed_by,
        changed_at
    )
    VALUES (
        txid_current(),
        NEW.id,
        v_changes,
        v_changed_by,
        v_changed_at
    )
    ON CONFLICT (txid, product_id) DO UPDATE
    SET
        product_changes = merge_audit_changes(product_audit_buffer.product_changes, EXCLUDED.product_changes),
        changed_by = EXCLUDED.changed_by,
        changed_at = EXCLUDED.changed_at;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_details_changes()
RETURNS TRIGGER AS $$
DECLARE
    v_changes    JSONB;
    v_product_id UUID;
    v_changed_by VARCHAR(200) := current_audit_actor();
    v_changed_at TIMESTAMPTZ := now();
BEGIN
    IF TG_OP = 'INSERT' THEN
        v_product_id := NEW.product_id;
        v_changes := jsonb_snapshot_diff(
            to_jsonb(NEW) - 'product_id',
            'new'
        );
    ELSIF TG_OP = 'DELETE' THEN
        v_product_id := OLD.product_id;
        v_changes := jsonb_snapshot_diff(
            to_jsonb(OLD) - 'product_id',
            'old'
        );
    ELSE
        v_product_id := NEW.product_id;
        v_changes := jsonb_row_diff(
            to_jsonb(OLD) - 'product_id',
            to_jsonb(NEW) - 'product_id'
        );
    END IF;

    IF v_changes = '{}'::jsonb THEN
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        END IF;

        RETURN NEW;
    END IF;

    INSERT INTO product_audit_buffer (
        txid,
        product_id,
        detail_changes,
        changed_by,
        changed_at
    )
    VALUES (
        txid_current(),
        v_product_id,
        v_changes,
        v_changed_by,
        v_changed_at
    )
    ON CONFLICT (txid, product_id) DO UPDATE
    SET
        detail_changes = merge_audit_changes(product_audit_buffer.detail_changes, EXCLUDED.detail_changes),
        changed_by = EXCLUDED.changed_by,
        changed_at = EXCLUDED.changed_at;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_relation_change(
    p_product_id UUID,
    p_relation   TEXT,
    p_item_id    INT,
    p_operation  TEXT
)
RETURNS VOID AS $$
DECLARE
    v_changed_by VARCHAR(200) := current_audit_actor();
    v_changed_at TIMESTAMPTZ := now();
BEGIN
    INSERT INTO product_audit_buffer (
        txid,
        product_id,
        changed_by,
        changed_at
    )
    VALUES (
        txid_current(),
        p_product_id,
        v_changed_by,
        v_changed_at
    )
    ON CONFLICT (txid, product_id) DO UPDATE
    SET
        changed_by = EXCLUDED.changed_by,
        changed_at = EXCLUDED.changed_at;

    IF p_relation = 'style_tags' THEN
        UPDATE product_audit_buffer
        SET
            style_tag_added = CASE
                WHEN p_operation = 'INSERT' AND p_item_id = ANY(style_tag_removed) THEN style_tag_added
                WHEN p_operation = 'INSERT' THEN append_unique_int(style_tag_added, p_item_id)
                ELSE array_remove(style_tag_added, p_item_id)
            END,
            style_tag_removed = CASE
                WHEN p_operation = 'INSERT' THEN array_remove(style_tag_removed, p_item_id)
                WHEN p_item_id = ANY(style_tag_added) THEN style_tag_removed
                ELSE append_unique_int(style_tag_removed, p_item_id)
            END,
            changed_by = v_changed_by,
            changed_at = v_changed_at
        WHERE txid = txid_current() AND product_id = p_product_id;
    ELSIF p_relation = 'vibe_tags' THEN
        UPDATE product_audit_buffer
        SET
            vibe_tag_added = CASE
                WHEN p_operation = 'INSERT' AND p_item_id = ANY(vibe_tag_removed) THEN vibe_tag_added
                WHEN p_operation = 'INSERT' THEN append_unique_int(vibe_tag_added, p_item_id)
                ELSE array_remove(vibe_tag_added, p_item_id)
            END,
            vibe_tag_removed = CASE
                WHEN p_operation = 'INSERT' THEN array_remove(vibe_tag_removed, p_item_id)
                WHEN p_item_id = ANY(vibe_tag_added) THEN vibe_tag_removed
                ELSE append_unique_int(vibe_tag_removed, p_item_id)
            END,
            changed_by = v_changed_by,
            changed_at = v_changed_at
        WHERE txid = txid_current() AND product_id = p_product_id;
    ELSIF p_relation = 'seasons' THEN
        UPDATE product_audit_buffer
        SET
            season_added = CASE
                WHEN p_operation = 'INSERT' AND p_item_id = ANY(season_removed) THEN season_added
                WHEN p_operation = 'INSERT' THEN append_unique_int(season_added, p_item_id)
                ELSE array_remove(season_added, p_item_id)
            END,
            season_removed = CASE
                WHEN p_operation = 'INSERT' THEN array_remove(season_removed, p_item_id)
                WHEN p_item_id = ANY(season_added) THEN season_removed
                ELSE append_unique_int(season_removed, p_item_id)
            END,
            changed_by = v_changed_by,
            changed_at = v_changed_at
        WHERE txid = txid_current() AND product_id = p_product_id;
    ELSE
        RAISE EXCEPTION 'Unsupported product relation for audit: %', p_relation;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_style_tag_changes()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM queue_product_relation_change(
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.product_id
            ELSE NEW.product_id
        END,
        'style_tags',
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.style_tag_id
            ELSE NEW.style_tag_id
        END,
        TG_OP
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_vibe_tag_changes()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM queue_product_relation_change(
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.product_id
            ELSE NEW.product_id
        END,
        'vibe_tags',
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.vibe_tag_id
            ELSE NEW.vibe_tag_id
        END,
        TG_OP
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION queue_product_season_changes()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM queue_product_relation_change(
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.product_id
            ELSE NEW.product_id
        END,
        'seasons',
        CASE
            WHEN TG_OP = 'DELETE' THEN OLD.season_id
            ELSE NEW.season_id
        END,
        TG_OP
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION build_product_analytics_snapshot(p_product_id UUID)
RETURNS JSONB AS $$
    SELECT
        jsonb_build_object(
            'product_id', p.id,
            'sku', p.sku,
            'name', p.name,
            'product_version', p.version,
            'status', p.status,
            'purchase_price', p.purchase_price,
            'currency', p.currency,
            'purchase_location_id', p.purchase_location_id,
            'purchase_location', pl.name,
            'image_count', p.image_count
        )
        || jsonb_build_object(
            'brand_id', b.id,
            'brand_name', b.name,
            'brand_tier', b.tier,
            'brand_country', b.country,
            'category_id', c.id,
            'category_name', c.name,
            'parent_category_id', cp.id,
            'parent_category', cp.name,
            'gender', c.gender,
            'product_type', c.product_type,
            'size_group', c.size_group
        )
        || jsonb_build_object(
            'material', pd.material,
            'condition', pd.condition,
            'color', pd.color,
            'year_of_release', pd.year_of_release,
            'is_vintage', pd.is_vintage,
            'is_collab', pd.is_collab,
            'collab_name', pd.collab_name,
            'is_limited_edition', pd.is_limited_edition,
            'size_value', pd.size_value,
            'size_value2', pd.size_value2,
            'size_system', pd.size_system,
            'measurement_cm', pd.measurement_cm
        )
        || jsonb_build_object(
            'fit', pd.fit,
            'shoe_width', pd.shoe_width,
            'insole_length_cm', pd.insole_length_cm,
            'width_cm', pd.width_cm,
            'height_cm', pd.height_cm,
            'depth_cm', pd.depth_cm,
            'handle_type', pd.handle_type,
            'bag_size_label', pd.bag_size_label,
            'metal', pd.metal,
            'stone', pd.stone,
            'clasp_type', pd.clasp_type
        )
        || jsonb_build_object(
            'style_tag_ids', COALESCE(styles.tag_ids, ARRAY[]::INT[]),
            'style_tags', COALESCE(styles.tag_names, ARRAY[]::TEXT[]),
            'vibe_tag_ids', COALESCE(vibes.tag_ids, ARRAY[]::INT[]),
            'vibe_tags', COALESCE(vibes.tag_names, ARRAY[]::TEXT[]),
            'season_ids', COALESCE(seasons.tag_ids, ARRAY[]::INT[]),
            'season_tags', COALESCE(seasons.tag_names, ARRAY[]::TEXT[]),
            'is_deleted', p.is_deleted,
            'deleted_at', p.deleted_at,
            'created_at', p.created_at,
            'updated_at', p.updated_at
        )
    FROM product p
    JOIN brand b ON p.brand_id = b.id
    JOIN category c ON p.category_id = c.id
    LEFT JOIN purchase_location pl ON p.purchase_location_id = pl.id
    LEFT JOIN category cp ON c.parent_id = cp.id
    LEFT JOIN product_details pd ON p.id = pd.product_id
    LEFT JOIN LATERAL (
        SELECT
            ARRAY_AGG(st.id ORDER BY st.name, st.id) AS tag_ids,
            ARRAY_AGG(st.name ORDER BY st.name, st.id) AS tag_names
        FROM product_style_tag pst
        JOIN style_tag st ON pst.style_tag_id = st.id
        WHERE pst.product_id = p.id
    ) styles ON true
    LEFT JOIN LATERAL (
        SELECT
            ARRAY_AGG(vt.id ORDER BY vt.name, vt.id) AS tag_ids,
            ARRAY_AGG(vt.name ORDER BY vt.name, vt.id) AS tag_names
        FROM product_vibe_tag pvt
        JOIN vibe_tag vt ON pvt.vibe_tag_id = vt.id
        WHERE pvt.product_id = p.id
    ) vibes ON true
    LEFT JOIN LATERAL (
        SELECT
            ARRAY_AGG(s.id ORDER BY s.name, s.id) AS tag_ids,
            ARRAY_AGG(s.name ORDER BY s.name, s.id) AS tag_names
        FROM product_season psn
        JOIN season s ON psn.season_id = s.id
        WHERE psn.product_id = p.id
    ) seasons ON true
    WHERE p.id = p_product_id;
$$ LANGUAGE sql STABLE;

COMMENT ON FUNCTION build_product_analytics_snapshot(UUID) IS 'Builds a flattened JSON snapshot of the product aggregate for outbox events and analytics bootstrap';

CREATE OR REPLACE FUNCTION flush_product_audit_buffer()
RETURNS TRIGGER AS $$
DECLARE
    v_audit             product_audit_buffer%ROWTYPE;
    v_changes           JSONB := '{}'::jsonb;
    v_payload           JSONB;
    v_snapshot          JSONB;
    v_history_version   INT;
    v_event_version     INT;
    v_event_type        VARCHAR(40);
    v_status_old        VARCHAR(20);
    v_status_new        VARCHAR(20);
    v_has_status_change BOOLEAN := false;
BEGIN
    SELECT *
    INTO v_audit
    FROM product_audit_buffer
    WHERE txid = NEW.txid
      AND product_id = NEW.product_id;

    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    IF v_audit.product_changes <> '{}'::jsonb THEN
        v_changes := v_changes || jsonb_build_object('product', v_audit.product_changes);
    END IF;

    IF v_audit.detail_changes <> '{}'::jsonb THEN
        v_changes := v_changes || jsonb_build_object('details', v_audit.detail_changes);
    END IF;

    IF cardinality(v_audit.style_tag_added) > 0 OR cardinality(v_audit.style_tag_removed) > 0 THEN
        v_changes := v_changes || jsonb_build_object(
            'style_tags',
            jsonb_build_object(
                'added', to_jsonb(v_audit.style_tag_added),
                'removed', to_jsonb(v_audit.style_tag_removed)
            )
        );
    END IF;

    IF cardinality(v_audit.vibe_tag_added) > 0 OR cardinality(v_audit.vibe_tag_removed) > 0 THEN
        v_changes := v_changes || jsonb_build_object(
            'vibe_tags',
            jsonb_build_object(
                'added', to_jsonb(v_audit.vibe_tag_added),
                'removed', to_jsonb(v_audit.vibe_tag_removed)
            )
        );
    END IF;

    IF cardinality(v_audit.season_added) > 0 OR cardinality(v_audit.season_removed) > 0 THEN
        v_changes := v_changes || jsonb_build_object(
            'seasons',
            jsonb_build_object(
                'added', to_jsonb(v_audit.season_added),
                'removed', to_jsonb(v_audit.season_removed)
            )
        );
    END IF;

    IF v_audit.created_in_tx THEN
        v_changes := jsonb_build_object('event', 'created') || v_changes;
    END IF;

    IF v_changes = '{}'::jsonb THEN
        DELETE FROM product_audit_buffer
        WHERE txid = v_audit.txid
          AND product_id = v_audit.product_id;

        RETURN NULL;
    END IF;

    SELECT version
    INTO v_history_version
    FROM product
    WHERE id = v_audit.product_id
    FOR UPDATE;

    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    IF v_audit.created_in_tx THEN
        v_event_type := 'product.created';
        v_event_version := v_history_version;
        v_has_status_change := true;
    ELSE
        UPDATE product
        SET version = version + 1
        WHERE id = v_audit.product_id
        RETURNING version INTO v_event_version;

        IF COALESCE((v_audit.product_changes -> 'is_deleted' ->> 'new')::BOOLEAN, false) THEN
            v_event_type := 'product.deleted';
            v_has_status_change := v_audit.product_changes ? 'status';
        ELSIF v_audit.product_changes ? 'status' THEN
            v_event_type := 'product.status_changed';
            v_has_status_change := true;
        ELSE
            v_event_type := 'product.updated';
        END IF;
    END IF;

    INSERT INTO product_history (
        product_id,
        version,
        changes,
        changed_by,
        changed_at
    )
    VALUES (
        v_audit.product_id,
        v_event_version,
        v_changes,
        v_audit.changed_by,
        v_audit.changed_at
    );

    v_snapshot := build_product_analytics_snapshot(v_audit.product_id);

    IF v_snapshot IS NULL THEN
        RAISE EXCEPTION 'Failed to build analytics snapshot for product %', v_audit.product_id;
    END IF;

    IF v_audit.created_in_tx THEN
        v_status_new := v_snapshot ->> 'status';
    ELSIF v_audit.product_changes ? 'status' THEN
        v_status_old := v_audit.product_changes -> 'status' ->> 'old';
        v_status_new := v_audit.product_changes -> 'status' ->> 'new';
    END IF;

    IF v_has_status_change THEN
        INSERT INTO product_status_history (
            product_id,
            product_version,
            old_status,
            new_status,
            changed_by,
            changed_at
        )
        VALUES (
            v_audit.product_id,
            v_event_version,
            v_status_old,
            v_status_new,
            v_audit.changed_by,
            v_audit.changed_at
        );
    END IF;

    v_payload := jsonb_build_object(
        'source_service', 'product-service',
        'event_type', v_event_type,
        'product_id', v_audit.product_id,
        'product_version', v_event_version,
        'occurred_at', v_audit.changed_at,
        'changed_by', v_audit.changed_by,
        'changes', v_changes,
        'snapshot', v_snapshot
    );

    IF v_has_status_change THEN
        v_payload := v_payload || jsonb_build_object(
            'status_transition',
            jsonb_build_object(
                'old_status', v_status_old,
                'new_status', v_status_new
            )
        );
    END IF;

    INSERT INTO product_outbox_event (
        product_id,
        event_type,
        product_version,
        payload,
        occurred_at
    )
    VALUES (
        v_audit.product_id,
        v_event_type,
        v_event_version,
        v_payload,
        v_audit.changed_at
    );

    DELETE FROM product_audit_buffer
    WHERE txid = v_audit.txid
      AND product_id = v_audit.product_id;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_category_product_type()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_gender VARCHAR(10);
    v_parent_type   VARCHAR(20);
    v_has_cycle     BOOLEAN := false;
BEGIN
    IF NEW.parent_id IS NULL THEN
        RETURN NEW;
    END IF;

    IF NEW.id IS NOT NULL AND NEW.parent_id = NEW.id THEN
        RAISE EXCEPTION 'Category cannot reference itself as parent (id=%)', NEW.id;
    END IF;

    SELECT gender, product_type
    INTO v_parent_gender, v_parent_type
    FROM category
    WHERE id = NEW.parent_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Parent category % does not exist', NEW.parent_id;
    END IF;

    IF v_parent_type IS DISTINCT FROM NEW.product_type THEN
        RAISE EXCEPTION 'Child category product_type (%) must match parent product_type (%)',
            NEW.product_type, v_parent_type;
    END IF;

    IF v_parent_gender IS DISTINCT FROM NEW.gender THEN
        RAISE EXCEPTION 'Child category gender (%) must match parent gender (%)',
            NEW.gender, v_parent_gender;
    END IF;

    IF NEW.id IS NOT NULL THEN
        WITH RECURSIVE ancestors AS (
            SELECT c.id, c.parent_id
            FROM category c
            WHERE c.id = NEW.parent_id
            UNION ALL
            SELECT c.id, c.parent_id
            FROM category c
            JOIN ancestors a ON c.id = a.parent_id
        )
        SELECT EXISTS (
            SELECT 1
            FROM ancestors
            WHERE id = NEW.id
        )
        INTO v_has_cycle;

        IF v_has_cycle THEN
            RAISE EXCEPTION 'Category cycle detected for category id=%', NEW.id;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_product_details_for_category()
RETURNS TRIGGER AS $$
DECLARE
    v_size_group   VARCHAR(20);
    v_product_type VARCHAR(20);
BEGIN
    SELECT c.size_group, c.product_type
    INTO v_size_group, v_product_type
    FROM product p
    JOIN category c ON c.id = p.category_id
    WHERE p.id = NEW.product_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Cannot validate product_details: product % not found', NEW.product_id;
    END IF;

    IF NEW.measurement_cm IS NOT NULL AND NEW.measurement_cm <= 0 THEN
        RAISE EXCEPTION 'measurement_cm must be > 0';
    END IF;

    IF NEW.insole_length_cm IS NOT NULL AND NEW.insole_length_cm <= 0 THEN
        RAISE EXCEPTION 'insole_length_cm must be > 0';
    END IF;

    IF NEW.width_cm IS NOT NULL AND NEW.width_cm <= 0 THEN
        RAISE EXCEPTION 'width_cm must be > 0';
    END IF;

    IF NEW.height_cm IS NOT NULL AND NEW.height_cm <= 0 THEN
        RAISE EXCEPTION 'height_cm must be > 0';
    END IF;

    IF NEW.depth_cm IS NOT NULL AND NEW.depth_cm <= 0 THEN
        RAISE EXCEPTION 'depth_cm must be > 0';
    END IF;

    IF NEW.is_collab AND (NEW.collab_name IS NULL OR btrim(NEW.collab_name) = '') THEN
        RAISE EXCEPTION 'collab_name is required when is_collab=true';
    END IF;

    IF NOT NEW.is_collab AND NEW.collab_name IS NOT NULL THEN
        RAISE EXCEPTION 'collab_name must be NULL when is_collab=false';
    END IF;

    IF v_size_group IN ('letter', 'letter_or_numeric', 'waist_length', 'shoe', 'ring')
       AND (NEW.size_value IS NULL OR btrim(NEW.size_value) = '') THEN
        RAISE EXCEPTION 'size_value is required for size_group=%', v_size_group;
    END IF;

    IF v_size_group = 'letter' THEN
        IF NEW.size_value2 IS NOT NULL OR NEW.size_system IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=letter allows only size_value';
        END IF;
    ELSIF v_size_group = 'letter_or_numeric' THEN
        IF NEW.size_value2 IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=letter_or_numeric does not allow size_value2 or measurement_cm';
        END IF;
    ELSIF v_size_group = 'waist_length' THEN
        IF NEW.size_system IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=waist_length does not allow size_system or measurement_cm';
        END IF;
    ELSIF v_size_group = 'shoe' THEN
        IF NEW.size_system NOT IN ('EU', 'US', 'UK') THEN
            RAISE EXCEPTION 'size_group=shoe requires size_system in (EU, US, UK)';
        END IF;
        IF NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=shoe does not allow measurement_cm';
        END IF;
    ELSIF v_size_group = 'ring' THEN
        IF NEW.size_system NOT IN ('EU', 'US', 'UK') THEN
            RAISE EXCEPTION 'size_group=ring requires size_system in (EU, US, UK)';
        END IF;
        IF NEW.size_value2 IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=ring does not allow size_value2 or measurement_cm';
        END IF;
    ELSIF v_size_group = 'measurement_cm' THEN
        IF NEW.measurement_cm IS NULL THEN
            RAISE EXCEPTION 'size_group=measurement_cm requires measurement_cm';
        END IF;
        IF NEW.size_value IS NOT NULL OR NEW.size_value2 IS NOT NULL OR NEW.size_system IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=measurement_cm does not allow size_value/size_value2/size_system';
        END IF;
    ELSIF v_size_group = 'dimensions' THEN
        IF NEW.width_cm IS NULL OR NEW.height_cm IS NULL OR NEW.depth_cm IS NULL THEN
            RAISE EXCEPTION 'size_group=dimensions requires width_cm, height_cm and depth_cm';
        END IF;
        IF NEW.size_value IS NOT NULL OR NEW.size_value2 IS NOT NULL OR NEW.size_system IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=dimensions does not allow size_value/size_value2/size_system/measurement_cm';
        END IF;
    ELSIF v_size_group = 'hat' THEN
        IF NEW.size_value2 IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=hat does not allow size_value2';
        END IF;
        IF (NEW.size_value IS NULL OR btrim(NEW.size_value) = '') AND NEW.measurement_cm IS NULL THEN
            RAISE EXCEPTION 'size_group=hat requires size_value or measurement_cm';
        END IF;
    ELSIF v_size_group = 'one_size' THEN
        IF NEW.size_value IS NOT NULL OR NEW.size_value2 IS NOT NULL OR NEW.size_system IS NOT NULL OR NEW.measurement_cm IS NOT NULL THEN
            RAISE EXCEPTION 'size_group=one_size does not allow sizing fields';
        END IF;
    ELSE
        RAISE EXCEPTION 'Unsupported size_group=%', v_size_group;
    END IF;

    IF v_product_type = 'clothing' THEN
        IF NEW.shoe_width IS NOT NULL
           OR NEW.insole_length_cm IS NOT NULL
           OR NEW.width_cm IS NOT NULL
           OR NEW.height_cm IS NOT NULL
           OR NEW.depth_cm IS NOT NULL
           OR NEW.handle_type IS NOT NULL
           OR NEW.bag_size_label IS NOT NULL
           OR NEW.metal IS NOT NULL
           OR NEW.stone IS NOT NULL
           OR NEW.clasp_type IS NOT NULL THEN
            RAISE EXCEPTION 'clothing details cannot contain footwear/bag/jewelry-only attributes';
        END IF;
    ELSIF v_product_type = 'footwear' THEN
        IF v_size_group <> 'shoe' THEN
            RAISE EXCEPTION 'footwear category must use size_group=shoe';
        END IF;

        IF NEW.fit IS NOT NULL
           OR NEW.width_cm IS NOT NULL
           OR NEW.height_cm IS NOT NULL
           OR NEW.depth_cm IS NOT NULL
           OR NEW.handle_type IS NOT NULL
           OR NEW.bag_size_label IS NOT NULL
           OR NEW.metal IS NOT NULL
           OR NEW.stone IS NOT NULL
           OR NEW.clasp_type IS NOT NULL THEN
            RAISE EXCEPTION 'footwear details cannot contain clothing/bag/jewelry-only attributes';
        END IF;
    ELSIF v_product_type = 'bags' THEN
        IF v_size_group <> 'dimensions' THEN
            RAISE EXCEPTION 'bags category must use size_group=dimensions';
        END IF;

        IF NEW.fit IS NOT NULL
           OR NEW.shoe_width IS NOT NULL
           OR NEW.insole_length_cm IS NOT NULL
           OR NEW.metal IS NOT NULL
           OR NEW.stone IS NOT NULL
           OR NEW.clasp_type IS NOT NULL THEN
            RAISE EXCEPTION 'bags details cannot contain clothing/footwear/jewelry-only attributes';
        END IF;
    ELSIF v_product_type = 'jewelry' THEN
        IF NEW.fit IS NOT NULL
           OR NEW.shoe_width IS NOT NULL
           OR NEW.insole_length_cm IS NOT NULL
           OR NEW.width_cm IS NOT NULL
           OR NEW.height_cm IS NOT NULL
           OR NEW.depth_cm IS NOT NULL
           OR NEW.handle_type IS NOT NULL
           OR NEW.bag_size_label IS NOT NULL THEN
            RAISE EXCEPTION 'jewelry details cannot contain clothing/footwear/bag-only attributes';
        END IF;
    ELSIF v_product_type = 'accessories' THEN
        IF NEW.fit IS NOT NULL
           OR NEW.shoe_width IS NOT NULL
           OR NEW.insole_length_cm IS NOT NULL
           OR NEW.width_cm IS NOT NULL
           OR NEW.height_cm IS NOT NULL
           OR NEW.depth_cm IS NOT NULL
           OR NEW.handle_type IS NOT NULL
           OR NEW.bag_size_label IS NOT NULL
           OR NEW.metal IS NOT NULL
           OR NEW.stone IS NOT NULL
           OR NEW.clasp_type IS NOT NULL THEN
            RAISE EXCEPTION 'accessories details cannot contain clothing/footwear/bag/jewelry-only attributes';
        END IF;
    ELSE
        RAISE EXCEPTION 'Unsupported product_type=%', v_product_type;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_product_status_transition()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NEW;
    END IF;

    IF (OLD.status = 'intake'      AND NEW.status IN ('inspection', 'rejected')) OR
       (OLD.status = 'inspection'  AND NEW.status IN ('preparation', 'rejected')) OR
       (OLD.status = 'rejected'    AND NEW.status IN ('preparation')) OR
       (OLD.status = 'preparation' AND NEW.status IN ('inspection', 'photo_queue')) OR
       (OLD.status = 'photo_queue' AND NEW.status IN ('preparation', 'photo_done')) OR
       (OLD.status = 'photo_done'  AND NEW.status IN ('photo_queue', 'ready')) OR
       (OLD.status = 'ready'       AND NEW.status IN ('reserved', 'sold')) OR
       (OLD.status = 'reserved'    AND NEW.status IN ('ready', 'sold')) OR
       (OLD.status = 'sold'        AND NEW.status IN ('returned')) OR
       (OLD.status = 'returned'    AND NEW.status IN ('preparation', 'ready')) THEN
        RETURN NEW;
    END IF;

    RAISE EXCEPTION 'Illegal status transition: % -> %', OLD.status, NEW.status;
END;
$$ LANGUAGE plpgsql;
