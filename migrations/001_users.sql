CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    phone           TEXT NOT NULL UNIQUE,
    public_key      TEXT NOT NULL UNIQUE,
    encrypted_keypair BYTEA NOT NULL,
    encryption_nonce  BYTEA NOT NULL,
    pin_hash        TEXT NOT NULL,
    kyc_tier        SMALLINT NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    daily_spent     DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    daily_spent_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_phone ON users (phone);
CREATE INDEX IF NOT EXISTS idx_users_public_key ON users (public_key);
