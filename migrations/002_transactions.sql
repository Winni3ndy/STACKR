CREATE TABLE IF NOT EXISTS transactions (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id             UUID NOT NULL REFERENCES users(id),
    idempotency_key     TEXT NOT NULL UNIQUE,
    tx_type             TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    amount              DOUBLE PRECISION NOT NULL,
    asset_code          TEXT NOT NULL DEFAULT 'USDC',
    stellar_tx_hash     TEXT,
    recipient_phone     TEXT,
    recipient_public_key TEXT,
    merchant_code       TEXT,
    memo                TEXT,
    error_detail        TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions (user_id);
CREATE INDEX IF NOT EXISTS idx_transactions_idempotency ON transactions (idempotency_key);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions (status);
CREATE INDEX IF NOT EXISTS idx_transactions_created ON transactions (created_at);
