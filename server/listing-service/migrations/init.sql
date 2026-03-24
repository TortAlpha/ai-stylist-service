-- ============================================================
-- Listing Service — PostgreSQL Init Script (v4)
-- Branded second-hand clothing store
--
-- Domain: Marketplace listings, pricing, reservations, sales tracking
-- References product_id from Product Service (no FK — cross-service)
-- ============================================================

BEGIN;

-- ============================================================
-- EXTENSIONS
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- MARKETPLACES
-- ============================================================

CREATE TABLE marketplace (
    id        SERIAL PRIMARY KEY,
    name      VARCHAR(200) NOT NULL UNIQUE,
    code      VARCHAR(20)  NOT NULL UNIQUE,
    base_url  TEXT,
    is_active BOOLEAN      NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE  marketplace          IS 'External marketplace directory (eBay, Etsy, Telegram, etc.)';
COMMENT ON COLUMN marketplace.code     IS 'Short code for internal use (e.g. EBAY, ETSY, TG, KP)';
COMMENT ON COLUMN marketplace.base_url IS 'Base URL of the marketplace (optional)';

INSERT INTO marketplace (name, code, base_url) VALUES
    ('Telegram',        'TG',   'https://t.me'),
    ('eBay',            'EBAY', 'https://www.ebay.com'),
    ('Etsy',            'ETSY', 'https://www.etsy.com'),
    ('KupujemProdajem', 'KP',   'https://www.kupujemprodajem.com');

-- ============================================================
-- PRODUCT LISTINGS
-- product_id is a soft reference to Product Service (no FK)
-- ============================================================

CREATE TABLE product_listing (
    id              SERIAL PRIMARY KEY,
    product_id      UUID         NOT NULL,
    marketplace_id  INT          NOT NULL REFERENCES marketplace(id) ON DELETE CASCADE,
    external_id     VARCHAR(500),
    external_url    TEXT,
    listing_price   DECIMAL(12, 2) NOT NULL CHECK (listing_price > 0),
    sold_price      DECIMAL(12, 2) CHECK (sold_price > 0),
    currency        VARCHAR(3)   NOT NULL DEFAULT 'EUR',
    status          VARCHAR(20)  NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'paused', 'sold', 'removed')),
    listed_at       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (product_id, marketplace_id)
);

CREATE INDEX idx_listing_product     ON product_listing (product_id);
CREATE INDEX idx_listing_marketplace ON product_listing (marketplace_id);
CREATE INDEX idx_listing_status      ON product_listing (status);
CREATE INDEX idx_listing_active      ON product_listing (product_id, status) WHERE status = 'active';

COMMENT ON TABLE  product_listing               IS 'Tracks where a product is listed — selling price, status, external links';
COMMENT ON COLUMN product_listing.product_id    IS 'References product in Product Service (no FK — cross-service boundary)';
COMMENT ON COLUMN product_listing.external_id   IS 'Product identifier on the external platform';
COMMENT ON COLUMN product_listing.external_url  IS 'Direct link to the listing on the external platform';
COMMENT ON COLUMN product_listing.listing_price IS 'Selling price on this marketplace';
COMMENT ON COLUMN product_listing.sold_price    IS 'Actual sale price after negotiation — filled when status = sold';
COMMENT ON COLUMN product_listing.currency      IS 'Currency for the listing/sold price';
COMMENT ON COLUMN product_listing.status        IS 'Listing lifecycle: active, paused, sold, removed';

-- ============================================================
-- LISTING HISTORY (audit log — JSONB diff)
-- ============================================================

CREATE TABLE listing_history (
    id          BIGSERIAL    PRIMARY KEY,
    listing_id  INT          NOT NULL REFERENCES product_listing(id) ON DELETE CASCADE,
    changes     JSONB        NOT NULL,
    changed_by  VARCHAR(200),
    changed_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_listing_history_listing ON listing_history (listing_id);

COMMENT ON TABLE  listing_history            IS 'Audit log for listing changes — price updates, status transitions';
COMMENT ON COLUMN listing_history.changes    IS '{"field": {"old": ..., "new": ...}}';
COMMENT ON COLUMN listing_history.changed_by IS 'Who made the change';

-- ============================================================
-- LISTING RESERVATION
-- Customer reserves through a specific listing on a marketplace
-- Service pauses all other listings for this product_id
-- and emits event → Product Service sets status = 'reserved'
-- ============================================================

CREATE TABLE listing_reservation (
    id              SERIAL       PRIMARY KEY,
    listing_id      INT          NOT NULL UNIQUE REFERENCES product_listing(id) ON DELETE CASCADE,
    product_id      UUID         NOT NULL,
    reserved_by     VARCHAR(200) NOT NULL,
    reserved_until  TIMESTAMPTZ  NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_lreservation_product ON listing_reservation (product_id);
CREATE INDEX idx_lreservation_until   ON listing_reservation (reserved_until);

COMMENT ON TABLE  listing_reservation                IS 'Active reservation — one per product, tied to the listing the customer came through';
COMMENT ON COLUMN listing_reservation.listing_id     IS 'Which specific listing the reservation was made through';
COMMENT ON COLUMN listing_reservation.product_id     IS 'Denormalized product_id for quick lookup across all listings';
COMMENT ON COLUMN listing_reservation.reserved_by    IS 'Customer identifier who placed the reservation';
COMMENT ON COLUMN listing_reservation.reserved_until IS 'Reservation expiry — on timeout, service releases reservation and resumes paused listings';

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

-- Track listing changes as JSONB diff
CREATE OR REPLACE FUNCTION track_listing_changes()
RETURNS TRIGGER AS $$
DECLARE
    v_changes JSONB := '{}';
BEGIN
    IF OLD.listing_price IS DISTINCT FROM NEW.listing_price THEN
        v_changes := v_changes || jsonb_build_object('listing_price',
            jsonb_build_object('old', OLD.listing_price, 'new', NEW.listing_price));
    END IF;

    IF OLD.sold_price IS DISTINCT FROM NEW.sold_price THEN
        v_changes := v_changes || jsonb_build_object('sold_price',
            jsonb_build_object('old', OLD.sold_price, 'new', NEW.sold_price));
    END IF;

    IF OLD.currency IS DISTINCT FROM NEW.currency THEN
        v_changes := v_changes || jsonb_build_object('currency',
            jsonb_build_object('old', OLD.currency, 'new', NEW.currency));
    END IF;

    IF OLD.status IS DISTINCT FROM NEW.status THEN
        v_changes := v_changes || jsonb_build_object('status',
            jsonb_build_object('old', OLD.status, 'new', NEW.status));
    END IF;

    IF OLD.external_id IS DISTINCT FROM NEW.external_id THEN
        v_changes := v_changes || jsonb_build_object('external_id',
            jsonb_build_object('old', OLD.external_id, 'new', NEW.external_id));
    END IF;

    IF OLD.external_url IS DISTINCT FROM NEW.external_url THEN
        v_changes := v_changes || jsonb_build_object('external_url',
            jsonb_build_object('old', OLD.external_url, 'new', NEW.external_url));
    END IF;

    IF v_changes = '{}' THEN
        RETURN NEW;
    END IF;

    INSERT INTO listing_history (listing_id, changes, changed_by, changed_at)
    VALUES (
        OLD.id,
        v_changes,
        COALESCE(current_setting('app.changed_by', true), current_user),
        now()
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- TRIGGERS
-- ============================================================

CREATE TRIGGER trg_listing_updated_at
    BEFORE UPDATE ON product_listing
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

CREATE TRIGGER trg_listing_history
    BEFORE UPDATE ON product_listing
    FOR EACH ROW EXECUTE FUNCTION track_listing_changes();

-- ============================================================
-- VIEWS
-- ============================================================

-- Active listings with marketplace info
CREATE VIEW v_active_listings AS
SELECT
    pl.id              AS listing_id,
    pl.product_id,
    m.name             AS marketplace_name,
    m.code             AS marketplace_code,
    pl.listing_price,
    pl.currency,
    pl.external_id,
    pl.external_url,
    lr.reserved_by,
    lr.reserved_until,
    pl.listed_at,
    pl.updated_at
FROM product_listing pl
JOIN marketplace m ON pl.marketplace_id = m.id
LEFT JOIN listing_reservation lr ON pl.id = lr.listing_id
WHERE pl.status = 'active';

COMMENT ON VIEW v_active_listings IS 'All currently active listings across marketplaces';

-- Sales summary per product
CREATE VIEW v_sales_summary AS
SELECT
    pl.product_id,
    m.name             AS marketplace_name,
    m.code             AS marketplace_code,
    pl.listing_price,
    pl.sold_price,
    pl.currency,
    pl.listed_at,
    pl.updated_at      AS sold_at
FROM product_listing pl
JOIN marketplace m ON pl.marketplace_id = m.id
WHERE pl.status = 'sold';

COMMENT ON VIEW v_sales_summary IS 'Completed sales — for profit/margin reporting';

COMMIT;