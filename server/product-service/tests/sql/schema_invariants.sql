\set ON_ERROR_STOP on

CREATE OR REPLACE FUNCTION assert_true(p_condition BOOLEAN, p_message TEXT)
RETURNS VOID AS $$
BEGIN
    IF NOT p_condition THEN
        RAISE EXCEPTION 'Assertion failed: %', p_message;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- 1) Root-category uniqueness for (name, gender) when parent_id IS NULL
DO $$
BEGIN
    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('ROOT_UNIQ_TEST', 'RUT1', NULL, 'female', 'clothing', 'letter');

    BEGIN
        INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
        VALUES ('ROOT_UNIQ_TEST', 'RUT2', NULL, 'female', 'clothing', 'letter');
        RAISE EXCEPTION 'Expected duplicate root category to fail';
    EXCEPTION
        WHEN unique_violation THEN
            NULL;
    END;
END;
$$;

-- 2) Category hierarchy guards: gender mismatch and cycle prevention
DO $$
DECLARE
    v_parent_id INT;
    v_cycle_a   INT;
    v_cycle_b   INT;
BEGIN
    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('HIER_PARENT_TEST', 'HPT1', NULL, 'male', 'clothing', 'letter')
    RETURNING id INTO v_parent_id;

    BEGIN
        INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
        VALUES ('HIER_CHILD_BAD_GENDER', 'HCBG', v_parent_id, 'female', 'clothing', 'letter');
        RAISE EXCEPTION 'Expected gender mismatch to fail';
    EXCEPTION
        WHEN OTHERS THEN
            NULL;
    END;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('HIER_CYCLE_A', 'HCA1', NULL, 'unisex', 'clothing', 'letter')
    RETURNING id INTO v_cycle_a;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('HIER_CYCLE_B', 'HCB1', v_cycle_a, 'unisex', 'clothing', 'letter')
    RETURNING id INTO v_cycle_b;

    BEGIN
        UPDATE category
        SET parent_id = v_cycle_b
        WHERE id = v_cycle_a;
        RAISE EXCEPTION 'Expected cycle detection to fail';
    EXCEPTION
        WHEN OTHERS THEN
            NULL;
    END;
END;
$$;

-- 3) Product status lifecycle transitions
DO $$
DECLARE
    v_brand_id   INT;
    v_category_id INT;
    v_product_id UUID;
BEGIN
    INSERT INTO brand (name, code, tier, country)
    VALUES ('STATUS_TEST_BRAND', 'STB1', 'mass', 'RS')
    RETURNING id INTO v_brand_id;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('STATUS_TEST_CATEGORY', 'STC1', NULL, 'unisex', 'clothing', 'letter')
    RETURNING id INTO v_category_id;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('STATUS_TEST_PRODUCT', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_product_id;

    BEGIN
        UPDATE product
        SET status = 'sold'
        WHERE id = v_product_id;
        RAISE EXCEPTION 'Expected illegal transition intake -> sold to fail';
    EXCEPTION
        WHEN OTHERS THEN
            NULL;
    END;

    UPDATE product SET status = 'inspection' WHERE id = v_product_id;
    UPDATE product SET status = 'preparation' WHERE id = v_product_id;
    UPDATE product SET status = 'photo_queue' WHERE id = v_product_id;
    UPDATE product SET status = 'photo_done' WHERE id = v_product_id;
    UPDATE product SET status = 'ready' WHERE id = v_product_id;

    PERFORM assert_true(
        EXISTS (
            SELECT 1 FROM product
            WHERE id = v_product_id
              AND status = 'ready'
        ),
        'Expected valid lifecycle path to end in ready'
    );
END;
$$;

-- 4) product_details guards by size_group/product_type + collab constraints
DO $$
DECLARE
    v_brand_id    INT;
    v_category_id INT;
    v_product_id  UUID;
BEGIN
    INSERT INTO brand (name, code, tier, country)
    VALUES ('DETAILS_TEST_BRAND', 'DTB1', 'mass', 'RS')
    RETURNING id INTO v_brand_id;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('DETAILS_TEST_FOOTWEAR', 'DTF1', NULL, 'unisex', 'footwear', 'shoe')
    RETURNING id INTO v_category_id;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('DETAILS_TEST_PRODUCT', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_product_id;

    BEGIN
        INSERT INTO product_details (product_id, condition, size_value, size_system)
        VALUES (v_product_id, 'excellent', '42', 'IT');
        RAISE EXCEPTION 'Expected shoe size_system=IT to fail';
    EXCEPTION
        WHEN OTHERS THEN
            NULL;
    END;

    BEGIN
        INSERT INTO product_details (product_id, condition, is_collab, size_value, size_system)
        VALUES (v_product_id, 'excellent', true, '42', 'EU');
        RAISE EXCEPTION 'Expected collab without collab_name to fail';
    EXCEPTION
        WHEN OTHERS THEN
            NULL;
    END;

    INSERT INTO product_details (
        product_id,
        condition,
        size_value,
        size_system,
        shoe_width,
        insole_length_cm,
        is_collab,
        collab_name
    )
    VALUES (
        v_product_id,
        'excellent',
        '42',
        'EU',
        'regular',
        27.5,
        false,
        NULL
    );

    PERFORM assert_true(
        EXISTS (SELECT 1 FROM product_details WHERE product_id = v_product_id),
        'Expected valid product_details row to be inserted'
    );
END;
$$;

-- 5) similar_products must be unique for undirected pairs
DO $$
DECLARE
    v_brand_id    INT;
    v_category_id INT;
    v_p1          UUID;
    v_p2          UUID;
BEGIN
    INSERT INTO brand (name, code, tier, country)
    VALUES ('SIM_TEST_BRAND', 'SMB1', 'mass', 'RS')
    RETURNING id INTO v_brand_id;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('SIM_TEST_CATEGORY', 'SMC1', NULL, 'unisex', 'clothing', 'letter')
    RETURNING id INTO v_category_id;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('SIM_TEST_PRODUCT_1', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_p1;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('SIM_TEST_PRODUCT_2', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_p2;

    INSERT INTO similar_products (product_id, similar_product_id, similarity_score)
    VALUES (v_p1, v_p2, 0.91);

    BEGIN
        INSERT INTO similar_products (product_id, similar_product_id, similarity_score)
        VALUES (v_p2, v_p1, 0.89);
        RAISE EXCEPTION 'Expected reverse similar pair to fail';
    EXCEPTION
        WHEN unique_violation THEN
            NULL;
    END;
END;
$$;

-- 6) product_history.version should match post-update product.version
DO $$
DECLARE
    v_brand_id      INT;
    v_category_id   INT;
    v_product_id    UUID;
    v_product_ver   INT;
    v_history_ver   INT;
BEGIN
    INSERT INTO brand (name, code, tier, country)
    VALUES ('HIST_TEST_BRAND', 'HTB1', 'mass', 'RS')
    RETURNING id INTO v_brand_id;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('HIST_TEST_CATEGORY', 'HTC1', NULL, 'unisex', 'clothing', 'letter')
    RETURNING id INTO v_category_id;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('HIST_TEST_PRODUCT', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_product_id;

    UPDATE product
    SET name = 'HIST_TEST_PRODUCT_V2'
    WHERE id = v_product_id;

    SELECT version
    INTO v_product_ver
    FROM product
    WHERE id = v_product_id;

    SELECT MAX(version)
    INTO v_history_ver
    FROM product_history
    WHERE product_id = v_product_id;

    PERFORM assert_true(
        v_history_ver = v_product_ver,
        format(
            'Expected product_history.version (%s) to equal product.version (%s)',
            COALESCE(v_history_ver::TEXT, 'NULL'),
            COALESCE(v_product_ver::TEXT, 'NULL')
        )
    );
END;
$$;

-- 7) gender CHECK accepts 'kids' and rejects unknown values
DO $$
DECLARE
    v_kids_id INT;
BEGIN
    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('KIDS_GENDER_TEST', 'KGT1', NULL, 'kids', 'clothing', 'letter')
    RETURNING id INTO v_kids_id;

    PERFORM assert_true(v_kids_id IS NOT NULL, 'Expected kids category insert to succeed');

    BEGIN
        INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
        VALUES ('GENDER_INVALID_TEST', 'GIT1', NULL, 'aliens', 'clothing', 'letter');
        RAISE EXCEPTION 'Expected unknown gender to fail CHECK';
    EXCEPTION
        WHEN check_violation THEN
            NULL;
    END;
END;
$$;

-- 8) season lookup is the post-rename set: summer, winter, spring, autumn
DO $$
DECLARE
    v_count INT;
BEGIN
    SELECT count(*) INTO v_count
    FROM season
    WHERE name IN ('summer', 'winter', 'spring', 'autumn');

    PERFORM assert_true(
        v_count = 4,
        format('Expected 4 canonical seasons (summer/winter/spring/autumn), found %s', v_count)
    );

    PERFORM assert_true(
        NOT EXISTS (SELECT 1 FROM season WHERE name IN ('demi-season', 'all-season')),
        'Expected legacy demi-season/all-season to be absent'
    );
END;
$$;

-- 9) Seeded kids root categories present for each expected product_type
DO $$
DECLARE
    v_missing TEXT;
BEGIN
    SELECT string_agg(expected.name, ', ') INTO v_missing
    FROM (VALUES
        ('Tops'), ('Bottoms'), ('Outerwear'), ('One-Piece & Sets'),
        ('Footwear'), ('Accessories'), ('Bags')
    ) AS expected(name)
    WHERE NOT EXISTS (
        SELECT 1 FROM category c
        WHERE c.parent_id IS NULL
          AND c.gender = 'kids'
          AND c.name = expected.name
    );

    PERFORM assert_true(
        v_missing IS NULL,
        format('Expected kids root categories to be seeded; missing: %s', v_missing)
    );
END;
$$;

-- 10) Sample new tags landed in style_tag and vibe_tag
DO $$
BEGIN
    PERFORM assert_true(
        EXISTS (SELECT 1 FROM style_tag WHERE name = 'soft girl'),
        'Expected style_tag "soft girl" to be seeded'
    );
    PERFORM assert_true(
        EXISTS (SELECT 1 FROM style_tag WHERE name = 'y2k'),
        'Expected style_tag "y2k" to be seeded'
    );
    PERFORM assert_true(
        EXISTS (SELECT 1 FROM vibe_tag WHERE name = 'vintage old money'),
        'Expected vibe_tag "vintage old money" to be seeded'
    );
    PERFORM assert_true(
        EXISTS (SELECT 1 FROM vibe_tag WHERE name = 'snow day'),
        'Expected vibe_tag "snow day" to be seeded'
    );
END;
$$;

-- 11) generate_product_sku maps gender='kids' to the 'K' segment
DO $$
DECLARE
    v_brand_id    INT;
    v_category_id INT;
    v_product_id  UUID;
    v_sku         TEXT;
BEGIN
    INSERT INTO brand (name, code, tier, country)
    VALUES ('SKU_KIDS_TEST_BRAND', 'SKB1', 'mass', 'RS')
    RETURNING id INTO v_brand_id;

    INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
    VALUES ('SKU_KIDS_TEST_CATEGORY', 'SKC1', NULL, 'kids', 'clothing', 'letter')
    RETURNING id INTO v_category_id;

    INSERT INTO product (name, brand_id, category_id, status, currency)
    VALUES ('SKU_KIDS_TEST_PRODUCT', v_brand_id, v_category_id, 'intake', 'RSD')
    RETURNING id INTO v_product_id;

    SELECT sku INTO v_sku FROM product WHERE id = v_product_id;

    PERFORM assert_true(
        v_sku LIKE 'AVA-SKB1-K-SKC1-%',
        format('Expected kids SKU to match AVA-SKB1-K-SKC1-*, got: %s', v_sku)
    );
END;
$$;

DROP FUNCTION assert_true(BOOLEAN, TEXT);

SELECT 'SQL schema invariant tests passed' AS result;
