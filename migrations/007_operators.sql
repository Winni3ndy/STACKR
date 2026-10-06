-- Operator table: each row = one tenant with its own config, wallets, and integrations
CREATE TABLE IF NOT EXISTS operators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- Fiat config
    fiat_currency TEXT NOT NULL DEFAULT 'RWF',
    fiat_country TEXT NOT NULL DEFAULT 'RW',

    -- Wallet derivation (per-operator master seed)
    wallet_master_seed TEXT NOT NULL,
    wallet_encryption_key TEXT NOT NULL,

    -- Stellar config
    fee_payer_secret TEXT NOT NULL,
    usdc_asset_code TEXT NOT NULL DEFAULT 'USDC',
    usdc_issuer TEXT NOT NULL,
    stellar_horizon_url TEXT NOT NULL DEFAULT 'https://horizon-testnet.stellar.org',
    stellar_network_passphrase TEXT NOT NULL DEFAULT 'Test SDF Network ; September 2015',

    -- Anchor (SEP-24) config
    anchor_domain TEXT NOT NULL DEFAULT 'cowrie.exchange',

    -- Africa's Talking credentials
    at_api_key TEXT NOT NULL DEFAULT '',
    at_username TEXT NOT NULL DEFAULT '',
    at_ussd_shortcode TEXT NOT NULL DEFAULT '*384#',
    at_sender_id TEXT NOT NULL DEFAULT 'STACKR',

    -- Airbills integration
    airbills_api_key TEXT NOT NULL DEFAULT '',
    airbills_base_url TEXT NOT NULL DEFAULT 'https://api.airbills.co',

    -- KYC daily limits (USDC equivalent)
    kyc_tier0_daily_limit DOUBLE PRECISION NOT NULL DEFAULT 50.0,
    kyc_tier1_daily_limit DOUBLE PRECISION NOT NULL DEFAULT 500.0,
    kyc_tier2_daily_limit DOUBLE PRECISION NOT NULL DEFAULT 5000.0,

    -- Webhook
    webhook_secret TEXT NOT NULL DEFAULT '',

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_operators_slug ON operators (slug);
CREATE INDEX IF NOT EXISTS idx_operators_shortcode ON operators (at_ussd_shortcode);
