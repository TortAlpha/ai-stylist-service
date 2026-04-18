-- ============================================================
-- TRIGGERS
-- ============================================================

CREATE TRIGGER trg_category_product_type
    BEFORE INSERT OR UPDATE ON category
    FOR EACH ROW EXECUTE FUNCTION validate_category_product_type();

CREATE TRIGGER trg_product_sku
    BEFORE INSERT ON product
    FOR EACH ROW EXECUTE FUNCTION generate_product_sku();

CREATE TRIGGER trg_product_created_audit
    AFTER INSERT ON product
    FOR EACH ROW EXECUTE FUNCTION queue_product_create();

CREATE TRIGGER trg_product_status_transition
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION validate_product_status_transition();

CREATE TRIGGER trg_product_updated_at
    BEFORE UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_product_version
    AFTER UPDATE ON product
    FOR EACH ROW EXECUTE FUNCTION queue_product_changes();

CREATE TRIGGER trg_product_details_version
    AFTER INSERT OR UPDATE OR DELETE ON product_details
    FOR EACH ROW EXECUTE FUNCTION queue_product_details_changes();

CREATE TRIGGER trg_product_details_guard
    BEFORE INSERT OR UPDATE ON product_details
    FOR EACH ROW EXECUTE FUNCTION validate_product_details_for_category();

CREATE TRIGGER trg_product_style_tag_version
    AFTER INSERT OR DELETE ON product_style_tag
    FOR EACH ROW EXECUTE FUNCTION queue_product_style_tag_changes();

CREATE TRIGGER trg_product_vibe_tag_version
    AFTER INSERT OR DELETE ON product_vibe_tag
    FOR EACH ROW EXECUTE FUNCTION queue_product_vibe_tag_changes();

CREATE TRIGGER trg_product_season_version
    AFTER INSERT OR DELETE ON product_season
    FOR EACH ROW EXECUTE FUNCTION queue_product_season_changes();

CREATE CONSTRAINT TRIGGER trg_product_audit_flush
    AFTER INSERT OR UPDATE ON product_audit_buffer
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION flush_product_audit_buffer();
