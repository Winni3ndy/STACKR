-- Anchor order tracking (following Payce's PAJ order pattern).
-- Tracks SEP-24 deposit/withdrawal orders from Stellar anchors.
CREATE TABLE IF NOT EXISTS anchor_orders (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id             UUID NOT NULL REFERENCES users(id),
    direction           TEXT NOT NULL,                    -- 'deposit' or 'withdrawal'
    status              TEXT NOT NULL DEFAULT 'pending',  -- pending | completed | error | expired
    anchor_domain       TEXT NOT NULL,                    -- e.g., 'cowrie.exchange'
    anchor_tx_id        TEXT,                             -- Anchor's transaction ID (from SEP-24)
    asset_code          TEXT NOT NULL DEFAULT 'USDC',
    amount              DOUBLE PRECISION,
    fiat_amount         DOUBLE PRECISION,
    fiat_currency       TEXT,                             -- NGN, KES, GHS
    stellar_tx_hash     TEXT,                             -- On-chain tx (for withdrawals we send)
    anchor_memo         TEXT,                             -- Memo for anchor payment
    anchor_account      TEXT,                             -- Anchor's Stellar address (for withdrawal)
    request_json        JSONB,                            -- Full request for audit
    response_json       JSONB,                            -- Full response for audit
    last_status_payload JSONB,                            -- Last status poll payload
    error_detail        TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Event log for every status change (audit trail, like Payce's paj_order_events)
CREATE TABLE IF NOT EXISTS anchor_order_events (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id    UUID NOT NULL REFERENCES anchor_orders(id),
    status      TEXT NOT NULL,
    payload     JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_anchor_orders_user ON anchor_orders (user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_anchor_orders_anchor_tx ON anchor_orders (anchor_tx_id);
CREATE INDEX IF NOT EXISTS idx_anchor_order_events_order ON anchor_order_events (order_id);
