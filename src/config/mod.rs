use std::env;
use std::net::IpAddr;
use std::str::FromStr;

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<AppConfig> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,

    // Database
    pub database_url: String,
    pub database_pool_max_size: usize,
    pub database_ssl: bool,

    // Redis
    pub redis_url: String,

    // Stellar
    pub stellar_horizon_url: String,
    pub stellar_network_passphrase: String,
    pub fee_payer_secret: String,
    pub usdc_asset_code: String,
    pub usdc_issuer: String,

    // Wallet derivation
    pub wallet_master_seed: String,
    pub wallet_encryption_key: String,

    // Africa's Talking
    pub at_api_key: String,
    pub at_username: String,
    pub at_ussd_shortcode: String,
    pub at_sender_id: String,

    // Airbills
    pub airbills_api_key: String,
    pub airbills_base_url: String,

    // Anchor (SEP-24 on/off-ramp)
    pub anchor_domain: String,
    pub public_base_url: String,

    // Security
    pub trusted_proxy_ips: Vec<IpAddr>,
    pub at_allowed_ips: Vec<IpAddr>,
    pub internal_api_key: String,
    pub webhook_secret: String,

    // Rate limiting
    pub rate_limit_requests: u64,
    pub rate_limit_window_secs: u64,

    // Local fiat currency (e.g., RWF, NGN, KES)
    pub fiat_currency: String,
    pub fiat_country: String,

    // KYC limits (in USDC equivalent)
    pub kyc_tier0_daily_limit: f64,
    pub kyc_tier1_daily_limit: f64,
    pub kyc_tier2_daily_limit: f64,

    // Environment
    pub environment: Environment,

    // Multi-tenant: slug of the default operator (for bootstrap & USSD fallback)
    pub default_operator_slug: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            _ => Environment::Development,
        };

        let config = Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("PORT must be a valid u16")?,

            database_url: env::var("DATABASE_URL").context("DATABASE_URL required")?,
            database_pool_max_size: env::var("DATABASE_POOL_MAX_SIZE")
                .unwrap_or_else(|_| "32".to_string())
                .parse()
                .context("DATABASE_POOL_MAX_SIZE must be a valid usize")?,
            database_ssl: env::var("DATABASE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),

            stellar_horizon_url: env::var("STELLAR_HORIZON_URL")
                .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string()),
            stellar_network_passphrase: env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
            fee_payer_secret: env::var("FEE_PAYER_SECRET").context("FEE_PAYER_SECRET required")?,
            usdc_asset_code: env::var("USDC_ASSET_CODE").unwrap_or_else(|_| "USDC".to_string()),
            usdc_issuer: env::var("USDC_ISSUER").context("USDC_ISSUER required")?,

            wallet_master_seed: env::var("WALLET_MASTER_SEED")
                .context("WALLET_MASTER_SEED required — losing this bricks every wallet")?,
            wallet_encryption_key: env::var("WALLET_ENCRYPTION_KEY").context(
                "WALLET_ENCRYPTION_KEY required — 64 hex chars (32 bytes) for AES-256-GCM",
            )?,

            at_api_key: env::var("AT_API_KEY").context("AT_API_KEY required (Africa's Talking)")?,
            at_username: env::var("AT_USERNAME")
                .context("AT_USERNAME required (Africa's Talking)")?,
            at_ussd_shortcode: env::var("AT_USSD_SHORTCODE")
                .unwrap_or_else(|_| "*384#".to_string()),
            at_sender_id: env::var("AT_SENDER_ID").unwrap_or_else(|_| "STACKR".to_string()),

            airbills_api_key: env::var("AIRBILLS_API_KEY").unwrap_or_else(|_| String::new()),
            airbills_base_url: env::var("AIRBILLS_BASE_URL")
                .unwrap_or_else(|_| "https://api.airbills.co".to_string()),

            fiat_currency: env::var("FIAT_CURRENCY").unwrap_or_else(|_| "RWF".to_string()),
            fiat_country: env::var("FIAT_COUNTRY").unwrap_or_else(|_| "RW".to_string()),

            anchor_domain: env::var("ANCHOR_DOMAIN")
                .unwrap_or_else(|_| "cowrie.exchange".to_string()),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            trusted_proxy_ips: Self::parse_ip_list(
                &env::var("TRUSTED_PROXY_IPS").unwrap_or_default(),
            ),
            at_allowed_ips: Self::parse_ip_list(&env::var("AT_ALLOWED_IPS").unwrap_or_default()),
            internal_api_key: env::var("INTERNAL_API_KEY").unwrap_or_default(),
            webhook_secret: env::var("WEBHOOK_SECRET").unwrap_or_default(),

            rate_limit_requests: env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            rate_limit_window_secs: env::var("RATE_LIMIT_WINDOW_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),

            kyc_tier0_daily_limit: env::var("KYC_TIER0_DAILY_LIMIT")
                .unwrap_or_else(|_| "50.0".to_string())
                .parse()
                .unwrap_or(50.0),
            kyc_tier1_daily_limit: env::var("KYC_TIER1_DAILY_LIMIT")
                .unwrap_or_else(|_| "500.0".to_string())
                .parse()
                .unwrap_or(500.0),
            kyc_tier2_daily_limit: env::var("KYC_TIER2_DAILY_LIMIT")
                .unwrap_or_else(|_| "5000.0".to_string())
                .parse()
                .unwrap_or(5000.0),

            default_operator_slug: env::var("DEFAULT_OPERATOR_SLUG").ok(),

            environment,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.wallet_encryption_key.len() != 64 {
            anyhow::bail!("WALLET_ENCRYPTION_KEY must be exactly 64 hex characters (32 bytes)");
        }
        hex::decode(&self.wallet_encryption_key)
            .context("WALLET_ENCRYPTION_KEY must be valid hex")?;

        if self.wallet_master_seed.is_empty() {
            anyhow::bail!("WALLET_MASTER_SEED cannot be empty");
        }

        if self.environment == Environment::Production {
            if self.webhook_secret.is_empty() {
                anyhow::bail!("WEBHOOK_SECRET required in production");
            }
            if self.internal_api_key.is_empty() {
                anyhow::bail!("INTERNAL_API_KEY required in production");
            }
        }

        Ok(())
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    fn parse_ip_list(s: &str) -> Vec<IpAddr> {
        s.split(',')
            .filter(|s| !s.trim().is_empty())
            .filter_map(|s| IpAddr::from_str(s.trim()).ok())
            .collect()
    }
}

pub fn global_config() -> &'static AppConfig {
    CONFIG.get().expect("AppConfig not initialized")
}

pub fn init_config(config: AppConfig) {
    CONFIG.set(config).expect("AppConfig already initialized");
}
