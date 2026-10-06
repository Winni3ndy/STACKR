use serde::Deserialize;
use tracing::warn;

use crate::context::RequestContext;
use crate::error::AppError;

/// Stellar Horizon account response (subset of fields we need)
#[derive(Debug, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub sequence: String,
    pub balances: Vec<Balance>,
}

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub balance: String,
    pub asset_type: String,
    #[serde(default)]
    pub asset_code: Option<String>,
    #[serde(default)]
    pub asset_issuer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub hash: String,
    pub successful: bool,
    #[serde(default)]
    pub result_xdr: Option<String>,
}

/// Fetch account details from Horizon
pub async fn get_account(
    ctx: &RequestContext,
    public_key: &str,
) -> Result<AccountResponse, AppError> {
    let url = format!(
        "{}/accounts/{}",
        ctx.operator.stellar_horizon_url, public_key
    );

    let response = ctx
        .state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    if response.status() == 404 {
        return Err(AppError::AccountNotFound(public_key.to_string()));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!(status = %status, body = %body, "Horizon error");
        return Err(AppError::StellarNetworkError(format!(
            "Horizon returned {status}"
        )));
    }

    response
        .json::<AccountResponse>()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))
}

/// Get all balances for an account as (asset_name, amount) pairs
pub async fn get_balances(
    ctx: &RequestContext,
    public_key: &str,
) -> Result<Vec<(String, f64)>, AppError> {
    let account = get_account(ctx, public_key).await?;

    let balances = account
        .balances
        .iter()
        .filter_map(|b| {
            let amount: f64 = b.balance.parse().ok()?;
            let name = match b.asset_type.as_str() {
                "native" => "XLM".to_string(),
                _ => b
                    .asset_code
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
            };
            Some((name, amount))
        })
        .collect();

    Ok(balances)
}

/// Submit a signed transaction envelope (base64 XDR) to Horizon
pub async fn submit_transaction(
    ctx: &RequestContext,
    tx_xdr_base64: &str,
) -> Result<TransactionResponse, AppError> {
    let url = format!("{}/transactions", ctx.operator.stellar_horizon_url);

    let response = ctx
        .state
        .http_client
        .post(&url)
        .form(&[("tx", tx_xdr_base64)])
        .send()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(body = %body, "Transaction submission failed");
        return Err(AppError::StellarSubmitFailed(body));
    }

    let tx_response: TransactionResponse = response
        .json()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    if !tx_response.successful {
        return Err(AppError::StellarSubmitFailed(
            tx_response
                .result_xdr
                .unwrap_or_else(|| "unknown error".to_string()),
        ));
    }

    Ok(tx_response)
}

/// Get the current sequence number for an account (needed for building transactions)
pub async fn get_sequence(ctx: &RequestContext, public_key: &str) -> Result<i64, AppError> {
    let account = get_account(ctx, public_key).await?;
    account
        .sequence
        .parse::<i64>()
        .map_err(|e| AppError::StellarNetworkError(format!("Invalid sequence: {e}")))
}

/// Get the current base fee from Horizon
pub async fn get_base_fee(ctx: &RequestContext) -> Result<u32, AppError> {
    #[derive(Deserialize)]
    struct FeeStats {
        last_ledger_base_fee: String,
    }

    let url = format!("{}/fee_stats", ctx.operator.stellar_horizon_url);

    let response = ctx
        .state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    if !response.status().is_success() {
        // Fallback to default base fee
        return Ok(100);
    }

    let stats: FeeStats = response
        .json()
        .await
        .map_err(|e| AppError::StellarNetworkError(e.to_string()))?;

    stats
        .last_ledger_base_fee
        .parse::<u32>()
        .map_err(|_| AppError::StellarNetworkError("Invalid fee".to_string()))
}
