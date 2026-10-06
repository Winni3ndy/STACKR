//! Price feed service for fiat ↔ USDC conversion (RWF, NGN, KES, etc.).
//! Uses CoinGecko API with Redis caching (60-second TTL).

use redis::AsyncCommands;

use tracing::{info, warn};

use crate::error::AppError;
use crate::AppState;

const CACHE_TTL_SECS: u64 = 60; // 1-minute cache for exchange rates
const COINGECKO_BASE: &str = "https://api.coingecko.com/api/v3";

/// Get the current USDC price in a fiat currency (e.g., NGN, KES, GHS).
/// Returns the amount of fiat per 1 USDC.
pub async fn usdc_to_fiat_rate(state: &AppState, fiat: &str) -> Result<f64, AppError> {
    let fiat_lower = fiat.to_lowercase();
    let cache_key = format!("rate:usdc:{}", fiat_lower);

    // Check Redis cache first
    let mut redis = state.redis.clone();
    if let Ok(Some(cached)) = redis.get::<_, Option<String>>(&cache_key).await {
        if let Ok(rate) = cached.parse::<f64>() {
            return Ok(rate);
        }
    }

    // Fetch from CoinGecko
    let rate = fetch_coingecko_rate(state, &fiat_lower)
        .await
        .map_err(|_| {
            warn!("CoinGecko failed, will use fallback");
            AppError::ExternalApiError {
                service: "price_feed".to_string(),
                detail: "All rate sources failed".to_string(),
            }
        })?;

    // Cache the result
    let _: Result<(), _> = redis
        .set_ex::<_, _, ()>(&cache_key, rate.to_string(), CACHE_TTL_SECS)
        .await;

    info!(fiat = %fiat_lower, rate = rate, "Fetched USDC/{} rate", fiat_lower.to_uppercase());

    Ok(rate)
}

/// Convert a fiat amount to USDC equivalent.
/// e.g., 1000 NGN at rate 1435 NGN/USDC = 0.697 USDC
pub async fn fiat_to_usdc(state: &AppState, fiat_amount: f64, fiat: &str) -> Result<f64, AppError> {
    let rate = usdc_to_fiat_rate(state, fiat).await?;
    if rate <= 0.0 {
        return Err(AppError::Internal("Invalid exchange rate".to_string()));
    }
    Ok(fiat_amount / rate)
}

/// Convert a USDC amount to fiat equivalent.
/// e.g., 1 USDC at rate 1435 NGN/USDC = 1435 NGN
pub async fn usdc_to_fiat(state: &AppState, usdc_amount: f64, fiat: &str) -> Result<f64, AppError> {
    let rate = usdc_to_fiat_rate(state, fiat).await?;
    Ok(usdc_amount * rate)
}

/// Fetch USDC price from CoinGecko.
/// Endpoint: /simple/price?ids=usd-coin&vs_currencies=ngn
async fn fetch_coingecko_rate(state: &AppState, fiat: &str) -> Result<f64, AppError> {
    let url = format!(
        "{}/simple/price?ids=usd-coin&vs_currencies={}",
        COINGECKO_BASE, fiat
    );

    let response = state
        .http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: format!("HTTP {}", response.status()),
        });
    }

    // Response format: { "usd-coin": { "ngn": 1435.68 } }
    let data: serde_json::Value =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: "CoinGecko".to_string(),
                detail: e.to_string(),
            })?;

    let rate = data
        .get("usd-coin")
        .and_then(|v| v.get(fiat))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: format!("No USDC/{} rate in response", fiat.to_uppercase()),
        })?;

    Ok(rate)
}

/// Get multiple rates at once (useful for displaying swap options).
pub async fn get_rates_batch(
    state: &AppState,
    fiat_currencies: &[&str],
) -> Result<Vec<(String, f64)>, AppError> {
    let vs = fiat_currencies
        .iter()
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join(",");

    let url = format!(
        "{}/simple/price?ids=usd-coin&vs_currencies={}",
        COINGECKO_BASE, vs
    );

    let response = state
        .http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: e.to_string(),
        })?;

    let data: serde_json::Value =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: "CoinGecko".to_string(),
                detail: e.to_string(),
            })?;

    let rates = fiat_currencies
        .iter()
        .filter_map(|fiat| {
            let rate = data.get("usd-coin")?.get(fiat.to_lowercase())?.as_f64()?;
            Some((fiat.to_uppercase(), rate))
        })
        .collect();

    Ok(rates)
}

/// Get the XLM/USD rate for native token conversions.
pub async fn xlm_usd_rate(state: &AppState) -> Result<f64, AppError> {
    let cache_key = "rate:xlm:usd";
    let mut redis = state.redis.clone();

    if let Ok(Some(cached)) = redis.get::<_, Option<String>>(cache_key).await {
        if let Ok(rate) = cached.parse::<f64>() {
            return Ok(rate);
        }
    }

    let url = format!(
        "{}/simple/price?ids=stellar&vs_currencies=usd",
        COINGECKO_BASE
    );

    let response = state
        .http_client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: e.to_string(),
        })?;

    let data: serde_json::Value =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: "CoinGecko".to_string(),
                detail: e.to_string(),
            })?;

    let rate = data
        .get("stellar")
        .and_then(|v| v.get("usd"))
        .and_then(|v| v.as_f64())
        .ok_or_else(|| AppError::ExternalApiError {
            service: "CoinGecko".to_string(),
            detail: "No XLM/USD rate".to_string(),
        })?;

    let _: Result<(), _> = redis
        .set_ex::<_, _, ()>(cache_key, rate.to_string(), CACHE_TTL_SECS)
        .await;

    Ok(rate)
}
