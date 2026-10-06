pub mod models;

use anyhow::{Context, Result};
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use tracing::info;

use crate::config::AppConfig;

pub fn create_pool(config: &AppConfig) -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.url = Some(config.database_url.clone());
    cfg.pool = Some(deadpool_postgres::PoolConfig {
        max_size: config.database_pool_max_size,
        ..Default::default()
    });

    let needs_tls = config.database_ssl
        || config.database_url.contains("sslmode=require")
        || config.database_url.contains("neon.tech");

    if needs_tls {
        info!("Creating database pool with TLS enabled");
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls_config);
        cfg.create_pool(Some(Runtime::Tokio1), tls)
            .context("Failed to create database pool with TLS")
    } else {
        info!("Creating database pool without TLS");
        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .context("Failed to create database pool")
    }
}

/// Apply SQL migrations in filename order. Idempotent — tracks applied migrations
/// in a `schema_migrations` table.
pub async fn run_migrations(client: &deadpool_postgres::Client) -> Result<()> {
    // Create migrations tracking table
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                filename TEXT PRIMARY KEY,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            &[],
        )
        .await
        .context("Failed to create schema_migrations table")?;

    // Read and apply migrations from embedded SQL
    let migrations = vec![
        (
            "001_users.sql",
            include_str!("../../migrations/001_users.sql"),
        ),
        (
            "002_transactions.sql",
            include_str!("../../migrations/002_transactions.sql"),
        ),
        (
            "003_user_activity.sql",
            include_str!("../../migrations/003_user_activity.sql"),
        ),
        (
            "004_merchants.sql",
            include_str!("../../migrations/004_merchants.sql"),
        ),
        (
            "005_sessions.sql",
            include_str!("../../migrations/005_sessions.sql"),
        ),
        (
            "006_anchor_orders.sql",
            include_str!("../../migrations/006_anchor_orders.sql"),
        ),
        (
            "007_operators.sql",
            include_str!("../../migrations/007_operators.sql"),
        ),
        (
            "008_api_keys.sql",
            include_str!("../../migrations/008_api_keys.sql"),
        ),
        (
            "009_add_operator_id.sql",
            include_str!("../../migrations/009_add_operator_id.sql"),
        ),
    ];

    for (filename, sql) in migrations {
        let applied: bool = client
            .query_one(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE filename = $1)",
                &[&filename],
            )
            .await?
            .get(0);

        if !applied {
            info!(migration = filename, "Applying migration");
            client
                .batch_execute(sql)
                .await
                .with_context(|| format!("Failed to apply migration: {filename}"))?;
            client
                .execute(
                    "INSERT INTO schema_migrations (filename) VALUES ($1)",
                    &[&filename],
                )
                .await?;
            info!(migration = filename, "Migration applied");
        }
    }

    info!("All migrations up to date");
    Ok(())
}
