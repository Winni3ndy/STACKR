use actix_web::{web, HttpResponse};
use redis::AsyncCommands;
use tracing::warn;

use crate::AppState;

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    checks: HealthChecks,
}

#[derive(serde::Serialize)]
struct HealthChecks {
    database: bool,
    redis: bool,
    stellar: bool,
}

pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    let db_ok = match state.db_pool.get().await {
        Ok(client) => client.execute("SELECT 1", &[]).await.is_ok(),
        Err(_) => false,
    };

    let redis_ok = state
        .redis
        .clone()
        .get::<_, Option<String>>("health")
        .await
        .is_ok();

    // Check Stellar Horizon
    let stellar_ok = state
        .http_client
        .get(format!("{}/", state.config.stellar_horizon_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    let all_ok = db_ok && redis_ok && stellar_ok;

    if !all_ok {
        warn!(
            db = db_ok,
            redis = redis_ok,
            stellar = stellar_ok,
            "Health check degraded"
        );
    }

    let response = HealthResponse {
        status: if all_ok { "healthy" } else { "degraded" },
        version: env!("CARGO_PKG_VERSION"),
        checks: HealthChecks {
            database: db_ok,
            redis: redis_ok,
            stellar: stellar_ok,
        },
    };

    if all_ok {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}
