use serde::Deserialize;
use stellar_xdr::curr::Asset;
use tracing::info;

use crate::context::RequestContext;
use crate::error::AppError;

use super::tx_builder;

/// Execute a swap via Stellar's PathPaymentStrictSend.
/// Uses the Stellar DEX order book — no external DEX needed.
/// Returns (tx_hash, amount_received).
pub async fn swap(
    ctx: &RequestContext,
    phone: &str,
    source_asset_code: &str,
    dest_asset_code: &str,
    send_amount: f64,
) -> Result<(String, f64), AppError> {
    let signing_key = super::wallet::load_signing_key(ctx, phone).await?;
    let public_key = super::wallet::public_key_to_stellar(&signing_key);

    // Resolve assets
    let (source_code, source_issuer) = resolve_asset_params(ctx, source_asset_code);
    let (dest_code, dest_issuer) = resolve_asset_params(ctx, dest_asset_code);

    let source_asset = tx_builder::build_asset(&source_code, &source_issuer)?;
    let dest_asset = tx_builder::build_asset(&dest_code, &dest_issuer)?;

    // Query Stellar path finding to get the best rate
    let paths = find_paths(
        ctx,
        &public_key,
        &source_code,
        &source_issuer,
        &dest_code,
        &dest_issuer,
        send_amount,
    )
    .await?;

    if paths.is_empty() {
        return Err(AppError::InvalidInput(
            "No swap path found for this pair/amount".to_string(),
        ));
    }

    // Use the best path (first result)
    let best_dest_amount: f64 = paths[0]
        .destination_amount
        .parse()
        .map_err(|_| AppError::Internal("Invalid path amount".to_string()))?;

    // Apply 0.5% slippage tolerance
    let dest_min = best_dest_amount * 0.995;

    // Build intermediate path assets
    let intermediate_path: Vec<Asset> = paths[0]
        .path
        .iter()
        .filter_map(|a| match a.asset_type.as_str() {
            "native" => Some(Asset::Native),
            _ => {
                let code = a.asset_code.as_deref()?;
                let issuer = a.asset_issuer.as_deref()?;
                tx_builder::build_asset(code, issuer).ok()
            }
        })
        .collect();

    // Build path payment operation
    let path_op = tx_builder::op_path_payment_strict_send(
        source_asset,
        tx_builder::to_stroops(send_amount),
        &public_key, // Send to self (swap)
        dest_asset,
        tx_builder::to_stroops(dest_min),
        intermediate_path,
    )?;

    // Build, sign, and submit
    let tx_hash = tx_builder::build_sign_submit(
        ctx,
        &signing_key,
        vec![path_op],
        tx_builder::memo_text("stackr-swap")?,
    )
    .await?;

    info!(
        phone = %phone,
        from = %source_asset_code,
        to = %dest_asset_code,
        sent = send_amount,
        min_receive = dest_min,
        tx_hash = %tx_hash,
        "Swap submitted"
    );

    // Return best_dest_amount as the expected received amount
    // In production, parse the transaction result XDR for the actual received amount
    Ok((tx_hash, best_dest_amount))
}

// ─── Path finding ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PathRecord {
    destination_amount: String,
    #[serde(default)]
    path: Vec<PathAsset>,
}

#[derive(Debug, Deserialize)]
struct PathAsset {
    asset_type: String,
    #[serde(default)]
    asset_code: Option<String>,
    #[serde(default)]
    asset_issuer: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PathResponse {
    #[serde(rename = "_embedded")]
    embedded: PathEmbedded,
}

#[derive(Debug, Deserialize)]
struct PathEmbedded {
    records: Vec<PathRecord>,
}

/// Query Stellar Horizon for payment paths (strict send)
async fn find_paths(
    ctx: &RequestContext,
    source_account: &str,
    source_code: &str,
    source_issuer: &str,
    dest_code: &str,
    dest_issuer: &str,
    amount: f64,
) -> Result<Vec<PathRecord>, AppError> {
    let mut url = format!(
        "{}/paths/strict-send?source_account={}&source_amount={:.7}",
        ctx.operator.stellar_horizon_url, source_account, amount
    );

    if source_code == "XLM" {
        url.push_str("&source_asset_type=native");
    } else {
        url.push_str(&format!(
            "&source_asset_type=credit_alphanum4&source_asset_code={}&source_asset_issuer={}",
            source_code, source_issuer
        ));
    }

    if dest_code == "XLM" {
        url.push_str("&destination_asset_type=native");
    } else {
        url.push_str(&format!(
            "&destination_asset_type=credit_alphanum4&destination_asset_code={}&destination_asset_issuer={}",
            dest_code, dest_issuer
        ));
    }

    let response = ctx
        .state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::StellarNetworkError(
            "Path finding failed".to_string(),
        ));
    }

    let path_response: PathResponse = response
        .json()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    Ok(path_response.embedded.records)
}

/// Resolve an asset name (e.g., "USDC", "XLM") to (code, issuer) pair
fn resolve_asset_params(ctx: &RequestContext, asset: &str) -> (String, String) {
    match asset {
        "XLM" | "native" => ("XLM".to_string(), String::new()),
        "USDC" => (
            ctx.operator.usdc_asset_code.clone(),
            ctx.operator.usdc_issuer.clone(),
        ),
        // TODO: Add USDT and other asset config
        _ => (asset.to_string(), String::new()),
    }
}
