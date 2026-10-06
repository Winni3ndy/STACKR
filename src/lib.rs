pub mod config;
pub mod context;
pub mod db;
pub mod error;
pub mod middleware;
pub mod operator;
pub mod routes;
pub mod services;
pub mod utils;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use deadpool_postgres::Pool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use config::AppConfig;

/// Shared application state passed to all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db_pool: Pool,
    pub redis: ConnectionManager,
    pub http_client: reqwest::Client,
}

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("stackr=debug,actix_web=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

pub async fn build_app_state() -> Result<AppState> {
    let config = AppConfig::from_env()?;

    // Database pool
    let db_pool = db::create_pool(&config)?;

    // Run migrations
    {
        let client = db_pool.get().await?;
        db::run_migrations(&client).await?;
    }

    // Redis connection manager (auto-reconnect)
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis = ConnectionManager::new(redis_client).await?;

    // Shared HTTP client with sensible defaults
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(20)
        .user_agent("Stackr/0.1.0")
        .build()?;

    let config = Arc::new(config);
    config::init_config((*config).clone());

    let state = AppState {
        config,
        db_pool,
        redis,
        http_client,
    };

    // Bootstrap default operator if none exists
    bootstrap_default_operator(&state).await?;

    Ok(state)
}

/// If no operators exist, auto-create a default operator from env vars
/// and backfill all existing rows with that operator_id.
async fn bootstrap_default_operator(state: &AppState) -> Result<()> {
    let db = state.db_pool.get().await?;

    let count: i64 = db
        .query_one("SELECT COUNT(*) FROM operators", &[])
        .await?
        .get(0);

    if count > 0 {
        info!("Operators table has {} entries, skipping bootstrap", count);
        return Ok(());
    }

    info!("No operators found — bootstrapping default operator from env vars");

    let config = &state.config;
    let slug = config.default_operator_slug.as_deref().unwrap_or("default");

    let row = db
        .query_one(
            "INSERT INTO operators (
                slug, name, is_active,
                fiat_currency, fiat_country,
                wallet_master_seed, wallet_encryption_key,
                fee_payer_secret, usdc_asset_code, usdc_issuer,
                stellar_horizon_url, stellar_network_passphrase,
                anchor_domain,
                at_api_key, at_username, at_ussd_shortcode, at_sender_id,
                airbills_api_key, airbills_base_url,
                kyc_tier0_daily_limit, kyc_tier1_daily_limit, kyc_tier2_daily_limit,
                webhook_secret
            ) VALUES (
                $1, $2, TRUE,
                $3, $4,
                $5, $6,
                $7, $8, $9,
                $10, $11,
                $12,
                $13, $14, $15, $16,
                $17, $18,
                $19, $20, $21,
                $22
            ) RETURNING id",
            &[
                &slug,
                &"Default Operator",
                &config.fiat_currency,
                &config.fiat_country,
                &config.wallet_master_seed,
                &config.wallet_encryption_key,
                &config.fee_payer_secret,
                &config.usdc_asset_code,
                &config.usdc_issuer,
                &config.stellar_horizon_url,
                &config.stellar_network_passphrase,
                &config.anchor_domain,
                &config.at_api_key,
                &config.at_username,
                &config.at_ussd_shortcode,
                &config.at_sender_id,
                &config.airbills_api_key,
                &config.airbills_base_url,
                &config.kyc_tier0_daily_limit,
                &config.kyc_tier1_daily_limit,
                &config.kyc_tier2_daily_limit,
                &config.webhook_secret,
            ],
        )
        .await?;

    let operator_id: uuid::Uuid = row.get("id");
    info!(operator_id = %operator_id, slug = %slug, "Default operator created");

    // Backfill existing rows with NULL operator_id
    let tables = ["users", "transactions", "merchants", "anchor_orders"];
    for table in tables {
        let query = format!(
            "UPDATE {} SET operator_id = $1 WHERE operator_id IS NULL",
            table
        );
        let updated = db.execute(&query, &[&operator_id]).await?;
        if updated > 0 {
            info!(table = %table, rows = updated, "Backfilled operator_id");
        }
    }

    // Invalidate operator cache so the new operator is available immediately
    operator::invalidate_cache();

    info!("Default operator bootstrap complete");
    Ok(())
}

pub async fn run_server(state: AppState, bind_addr: &str) -> Result<()> {
    let is_production = state.config.is_production();
    let state_data = web::Data::new(state);

    HttpServer::new(move || {
        let cors = if is_production {
            actix_cors::Cors::default()
                .allowed_origin("https://usestackr.xyz")
                .allowed_origin("https://www.usestackr.xyz")
                .allowed_origin("https://api.usestackr.xyz")
                .allowed_origin("https://developers.usestackr.xyz")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::AUTHORIZATION,
                    actix_web::http::header::CONTENT_TYPE,
                    actix_web::http::header::ACCEPT,
                ])
                .max_age(3600)
        } else {
            // Permissive in development for USSD simulator
            actix_cors::Cors::permissive()
        };

        App::new()
            .app_data(state_data.clone())
            // CORS (must be outermost wrap)
            .wrap(cors)
            // Observability
            .wrap(tracing_actix_web::TracingLogger::default())
            // Security headers
            .wrap(middleware::security_headers::SecurityHeaders)
            // Routes
            .configure(routes::configure)
    })
    .bind(bind_addr)?
    .workers(num_cpus::get())
    .shutdown_timeout(30)
    .run()
    .await?;

    Ok(())
}
