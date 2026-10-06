CREATE TABLE IF NOT EXISTS merchants (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    merchant_code   TEXT NOT NULL UNIQUE,
    name            TEXT NOT NULL,
    public_key      TEXT NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_merchants_code ON merchants (merchant_code);
