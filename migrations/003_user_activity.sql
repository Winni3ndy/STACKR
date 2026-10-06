-- Unified activity ledger for analytics (Metabase-friendly schema).
-- Writes are fire-and-forget via tokio::spawn so USSD latency is unaffected.
CREATE TABLE IF NOT EXISTS user_activity (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id),
    ref_id      TEXT NOT NULL,
    action      TEXT NOT NULL,
    detail      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_activity_ref ON user_activity (ref_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_user ON user_activity (user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_action ON user_activity (action);
CREATE INDEX IF NOT EXISTS idx_user_activity_created ON user_activity (created_at);
