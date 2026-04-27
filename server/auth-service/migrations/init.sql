BEGIN;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    refresh_token VARCHAR(512) NOT NULL UNIQUE,
    previous_refresh_token VARCHAR(512),
    previous_rotated_at TIMESTAMPTZ,
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    device_type VARCHAR(50) NOT NULL DEFAULT 'unknown',
    last_activity_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_refresh_token ON sessions (refresh_token);
CREATE INDEX idx_sessions_previous_refresh_token ON sessions (previous_refresh_token);

COMMIT;