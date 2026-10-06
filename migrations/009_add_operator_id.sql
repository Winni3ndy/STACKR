-- Add operator_id to tenant-scoped tables
ALTER TABLE users ADD COLUMN IF NOT EXISTS operator_id UUID REFERENCES operators(id);
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS operator_id UUID REFERENCES operators(id);
ALTER TABLE merchants ADD COLUMN IF NOT EXISTS operator_id UUID REFERENCES operators(id);
ALTER TABLE anchor_orders ADD COLUMN IF NOT EXISTS operator_id UUID REFERENCES operators(id);

-- Indexes for operator-scoped lookups
CREATE INDEX IF NOT EXISTS idx_users_operator ON users (operator_id);
CREATE INDEX IF NOT EXISTS idx_transactions_operator ON transactions (operator_id);
CREATE INDEX IF NOT EXISTS idx_merchants_operator ON merchants (operator_id);
CREATE INDEX IF NOT EXISTS idx_anchor_orders_operator ON anchor_orders (operator_id);

-- Composite unique: same phone can exist under different operators
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_operator_phone ON users (operator_id, phone);

-- Drop the old phone-only unique constraint (allow same phone across operators)
-- We use DO $$ to handle the case where the constraint might not exist
DO $$
BEGIN
    ALTER TABLE users DROP CONSTRAINT IF EXISTS users_phone_key;
EXCEPTION
    WHEN undefined_object THEN NULL;
END $$;
