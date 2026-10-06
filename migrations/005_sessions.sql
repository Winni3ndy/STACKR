-- USSD session state overflow. Primary session store is Redis;
-- this table catches sessions that outlive Redis TTL or need audit.
CREATE TABLE IF NOT EXISTS ussd_sessions (
    session_id      TEXT PRIMARY KEY,
    phone           TEXT NOT NULL,
    state           TEXT NOT NULL DEFAULT 'main_menu',
    data            JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_phone ON ussd_sessions (phone);
