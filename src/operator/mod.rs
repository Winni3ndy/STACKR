pub mod cli;

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use tracing::info;
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

/// Per-operator configuration loaded from the `operators` table.
/// Mirrors the columns so services can read operator-specific settings.
#[derive(Debug, Clone)]
pub struct OperatorConfig {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub is_active: bool,

    // Fiat
    pub fiat_currency: String,
    pub fiat_country: String,

    // Wallet derivation
    pub wallet_master_seed: String,
    pub wallet_encryption_key: String,

    // Stellar
    pub fee_payer_secret: String,
    pub usdc_asset_code: String,
    pub usdc_issuer: String,
    pub stellar_horizon_url: String,
    pub stellar_network_passphrase: String,

    // Anchor
    pub anchor_domain: String,

    // Africa's Talking
    pub at_api_key: String,
    pub at_username: String,
    pub at_ussd_shortcode: String,
    pub at_sender_id: String,

    // Airbills
    pub airbills_api_key: String,
    pub airbills_base_url: String,

    // KYC limits
    pub kyc_tier0_daily_limit: f64,
    pub kyc_tier1_daily_limit: f64,
    pub kyc_tier2_daily_limit: f64,

    // Webhook
    pub webhook_secret: String,
}

struct CacheEntry {
    config: std::sync::Arc<OperatorConfig>,
    loaded_at: Instant,
}

static CACHE: RwLock<Option<OperatorCache>> = RwLock::new(None);
const CACHE_TTL: Duration = Duration::from_secs(60);

struct OperatorCache {
    by_id: HashMap<Uuid, CacheEntry>,
    by_slug: HashMap<String, Uuid>,
    by_shortcode: HashMap<String, Uuid>,
}

impl OperatorCache {
    fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            by_slug: HashMap::new(),
            by_shortcode: HashMap::new(),
        }
    }
}

fn is_fresh(entry: &CacheEntry) -> bool {
    entry.loaded_at.elapsed() < CACHE_TTL
}

/// Invalidate the entire operator cache (e.g., after bootstrap or operator creation).
pub fn invalidate_cache() {
    if let Ok(mut cache) = CACHE.write() {
        *cache = None;
    }
}

fn insert_into_cache(config: &std::sync::Arc<OperatorConfig>) {
    if let Ok(mut guard) = CACHE.write() {
        let cache = guard.get_or_insert_with(OperatorCache::new);
        cache.by_slug.insert(config.slug.clone(), config.id);
        cache
            .by_shortcode
            .insert(config.at_ussd_shortcode.clone(), config.id);
        cache.by_id.insert(
            config.id,
            CacheEntry {
                config: config.clone(),
                loaded_at: Instant::now(),
            },
        );
    }
}

fn get_cached_by_id(id: &Uuid) -> Option<std::sync::Arc<OperatorConfig>> {
    let guard = CACHE.read().ok()?;
    let cache = guard.as_ref()?;
    let entry = cache.by_id.get(id)?;
    if is_fresh(entry) {
        Some(entry.config.clone())
    } else {
        None
    }
}

fn get_cached_id_by_slug(slug: &str) -> Option<Uuid> {
    let guard = CACHE.read().ok()?;
    let cache = guard.as_ref()?;
    cache.by_slug.get(slug).copied()
}

fn get_cached_id_by_shortcode(shortcode: &str) -> Option<Uuid> {
    let guard = CACHE.read().ok()?;
    let cache = guard.as_ref()?;
    cache.by_shortcode.get(shortcode).copied()
}

fn row_to_config(row: &tokio_postgres::Row) -> OperatorConfig {
    OperatorConfig {
        id: row.get("id"),
        slug: row.get("slug"),
        name: row.get("name"),
        is_active: row.get("is_active"),
        fiat_currency: row.get("fiat_currency"),
        fiat_country: row.get("fiat_country"),
        wallet_master_seed: row.get("wallet_master_seed"),
        wallet_encryption_key: row.get("wallet_encryption_key"),
        fee_payer_secret: row.get("fee_payer_secret"),
        usdc_asset_code: row.get("usdc_asset_code"),
        usdc_issuer: row.get("usdc_issuer"),
        stellar_horizon_url: row.get("stellar_horizon_url"),
        stellar_network_passphrase: row.get("stellar_network_passphrase"),
        anchor_domain: row.get("anchor_domain"),
        at_api_key: row.get("at_api_key"),
        at_username: row.get("at_username"),
        at_ussd_shortcode: row.get("at_ussd_shortcode"),
        at_sender_id: row.get("at_sender_id"),
        airbills_api_key: row.get("airbills_api_key"),
        airbills_base_url: row.get("airbills_base_url"),
        kyc_tier0_daily_limit: row.get("kyc_tier0_daily_limit"),
        kyc_tier1_daily_limit: row.get("kyc_tier1_daily_limit"),
        kyc_tier2_daily_limit: row.get("kyc_tier2_daily_limit"),
        webhook_secret: row.get("webhook_secret"),
    }
}

/// Load operator by ID (cached with 60s TTL).
pub async fn load_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<std::sync::Arc<OperatorConfig>, AppError> {
    if let Some(cached) = get_cached_by_id(&id) {
        return Ok(cached);
    }

    let db = state.db_pool.get().await?;
    let row = db
        .query_opt(
            "SELECT * FROM operators WHERE id = $1 AND is_active = TRUE",
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Operator {id} not found")))?;

    let config = std::sync::Arc::new(row_to_config(&row));
    insert_into_cache(&config);
    Ok(config)
}

/// Load operator by slug (cached with 60s TTL).
pub async fn load_by_slug(
    state: &AppState,
    slug: &str,
) -> Result<std::sync::Arc<OperatorConfig>, AppError> {
    if let Some(id) = get_cached_id_by_slug(slug) {
        if let Some(cached) = get_cached_by_id(&id) {
            return Ok(cached);
        }
    }

    let db = state.db_pool.get().await?;
    let row = db
        .query_opt(
            "SELECT * FROM operators WHERE slug = $1 AND is_active = TRUE",
            &[&slug],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Operator '{slug}' not found")))?;

    let config = std::sync::Arc::new(row_to_config(&row));
    insert_into_cache(&config);
    Ok(config)
}

/// Load operator by USSD shortcode (cached with 60s TTL).
pub async fn load_by_shortcode(
    state: &AppState,
    shortcode: &str,
) -> Result<std::sync::Arc<OperatorConfig>, AppError> {
    if let Some(id) = get_cached_id_by_shortcode(shortcode) {
        if let Some(cached) = get_cached_by_id(&id) {
            return Ok(cached);
        }
    }

    let db = state.db_pool.get().await?;
    let row = db
        .query_opt(
            "SELECT * FROM operators WHERE at_ussd_shortcode = $1 AND is_active = TRUE",
            &[&shortcode],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("No operator for shortcode '{shortcode}'")))?;

    let config = std::sync::Arc::new(row_to_config(&row));
    insert_into_cache(&config);
    Ok(config)
}

/// Load the default operator (first active operator, or by slug).
pub async fn load_default(state: &AppState) -> Result<std::sync::Arc<OperatorConfig>, AppError> {
    let slug = state
        .config
        .default_operator_slug
        .as_deref()
        .unwrap_or("default");

    match load_by_slug(state, slug).await {
        Ok(op) => Ok(op),
        Err(_) => {
            // Fallback: load the first active operator
            let db = state.db_pool.get().await?;
            let row = db
                .query_opt(
                    "SELECT * FROM operators WHERE is_active = TRUE ORDER BY created_at LIMIT 1",
                    &[],
                )
                .await?
                .ok_or_else(|| AppError::Internal("No active operators configured".to_string()))?;

            let config = std::sync::Arc::new(row_to_config(&row));
            insert_into_cache(&config);
            info!(operator = %config.slug, "Loaded default operator (fallback)");
            Ok(config)
        }
    }
}
