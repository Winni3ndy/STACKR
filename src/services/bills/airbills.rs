use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::context::RequestContext;
use crate::error::AppError;

#[derive(Debug, Serialize)]
struct AirtimePurchaseRequest {
    phone: String,
    amount: f64,
    product_type: String,
    country: String,
}

#[derive(Debug, Deserialize)]
struct AirbillsResponse {
    pub status: String,
    pub reference: Option<String>,
    pub message: Option<String>,
}

/// Purchase airtime or data for a phone number via Airbills API.
/// Converts local fiat to USDC, debits user's Stellar wallet, then calls Airbills.
pub async fn purchase_airtime(
    ctx: &RequestContext,
    user_phone: &str,
    target_phone: &str,
    product_type: &str, // "airtime" or "data"
    amount_fiat: f64,
) -> Result<String, AppError> {
    // Step 1: Convert fiat to USDC equivalent
    let currency = &ctx.operator.fiat_currency;
    let usdc_amount =
        crate::services::price_feed::fiat_to_usdc(&ctx.state, amount_fiat, currency).await?;

    // Step 2: Debit USDC from user — send to fee payer (platform treasury)
    let fee_payer_public = crate::services::stellar::wallet::fee_payer_public_key(ctx)?;
    crate::services::stellar::transfer::send_usdc(ctx, user_phone, &fee_payer_public, usdc_amount)
        .await
        .map_err(|e| {
            warn!(error = %e, phone = %user_phone, "Failed to debit USDC for airtime");
            e
        })?;

    // Step 3: Use configured country or detect from phone
    let country = ctx.operator.fiat_country.clone();

    // Step 4: Call Airbills API
    let url = format!("{}/v1/topup", ctx.operator.airbills_base_url);

    let request = AirtimePurchaseRequest {
        phone: target_phone.to_string(),
        amount: amount_fiat,
        product_type: product_type.to_string(),
        country,
    };

    let response = ctx
        .state
        .http_client
        .post(&url)
        .header(
            "Authorization",
            format!("Bearer {}", ctx.operator.airbills_api_key),
        )
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "Airbills".to_string(),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!(status = %status, body = %body, "Airbills API error");
        // Refund the user since API call failed
        refund_user(ctx, user_phone, usdc_amount, "airtime_refund").await;
        return Err(AppError::ExternalApiError {
            service: "Airbills".to_string(),
            detail: format!("HTTP {status}"),
        });
    }

    let result: AirbillsResponse =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: "Airbills".to_string(),
                detail: e.to_string(),
            })?;

    match result.status.as_str() {
        "success" => {
            let ref_id = result
                .reference
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            info!(
                user = %user_phone,
                target = %target_phone,
                product = %product_type,
                amount_fiat = amount_fiat,
                usdc_debited = usdc_amount,
                ref_id = %ref_id,
                "Airtime purchase successful"
            );
            Ok(ref_id)
        }
        _ => {
            // Refund the user since purchase failed
            refund_user(ctx, user_phone, usdc_amount, "airtime_refund").await;
            Err(AppError::ExternalApiError {
                service: "Airbills".to_string(),
                detail: result
                    .message
                    .unwrap_or_else(|| "Purchase failed".to_string()),
            })
        }
    }
}

/// Pay a utility bill via Airbills API.
/// Converts local fiat to USDC, debits user's Stellar wallet, then calls Airbills.
pub async fn pay_bill(
    ctx: &RequestContext,
    user_phone: &str,
    bill_type: &str,
    account_number: &str,
    amount_fiat: f64,
) -> Result<String, AppError> {
    // Step 1: Convert fiat to USDC equivalent
    let currency = &ctx.operator.fiat_currency;
    let usdc_amount =
        crate::services::price_feed::fiat_to_usdc(&ctx.state, amount_fiat, currency).await?;

    // Step 2: Debit USDC from user — send to fee payer (platform treasury)
    let fee_payer_public = crate::services::stellar::wallet::fee_payer_public_key(ctx)?;
    crate::services::stellar::transfer::send_usdc(ctx, user_phone, &fee_payer_public, usdc_amount)
        .await
        .map_err(|e| {
            warn!(error = %e, phone = %user_phone, "Failed to debit USDC for bill");
            e
        })?;

    // Step 3: Call Airbills bill payment API
    let url = format!("{}/v1/bills/pay", ctx.operator.airbills_base_url);

    let request = serde_json::json!({
        "type": bill_type,
        "account": account_number,
        "amount": amount_fiat,
        "country": ctx.operator.fiat_country,
    });

    let response = ctx
        .state
        .http_client
        .post(&url)
        .header(
            "Authorization",
            format!("Bearer {}", ctx.operator.airbills_api_key),
        )
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "Airbills".to_string(),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        refund_user(ctx, user_phone, usdc_amount, "bill_refund").await;
        return Err(AppError::ExternalApiError {
            service: "Airbills".to_string(),
            detail: body,
        });
    }

    let result: AirbillsResponse =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: "Airbills".to_string(),
                detail: e.to_string(),
            })?;

    match result.status.as_str() {
        "success" => {
            let ref_id = result
                .reference
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            info!(
                user = %user_phone,
                bill = %bill_type,
                account = %account_number,
                amount_fiat = amount_fiat,
                usdc_debited = usdc_amount,
                ref_id = %ref_id,
                "Bill payment successful"
            );
            Ok(ref_id)
        }
        _ => {
            refund_user(ctx, user_phone, usdc_amount, "bill_refund").await;
            Err(AppError::ExternalApiError {
                service: "Airbills".to_string(),
                detail: result
                    .message
                    .unwrap_or_else(|| "Payment failed".to_string()),
            })
        }
    }
}

/// Best-effort refund: send USDC back from fee payer to user if the API call fails.
/// Fire-and-forget — if refund fails, it gets logged for manual resolution.
async fn refund_user(ctx: &RequestContext, user_phone: &str, usdc_amount: f64, reason: &str) {
    let user_public_key = match get_user_public_key(ctx, user_phone).await {
        Some(k) => k,
        None => {
            warn!(phone = %user_phone, reason = %reason, "Cannot refund: user public key not found");
            return;
        }
    };

    // Send from fee payer back to user
    match crate::services::stellar::fee_payer::send_from_fee_payer(
        ctx,
        &user_public_key,
        usdc_amount,
    )
    .await
    {
        Ok(tx_hash) => {
            info!(
                phone = %user_phone,
                amount = usdc_amount,
                reason = %reason,
                tx_hash = %tx_hash,
                "Refund sent successfully"
            );
        }
        Err(e) => {
            // Critical: manual intervention needed
            warn!(
                phone = %user_phone,
                amount = usdc_amount,
                reason = %reason,
                error = %e,
                "REFUND FAILED — requires manual resolution"
            );
        }
    }
}

async fn get_user_public_key(ctx: &RequestContext, phone: &str) -> Option<String> {
    let db = ctx.state.db_pool.get().await.ok()?;
    let row = db
        .query_opt(
            "SELECT public_key FROM users WHERE phone = $1 AND operator_id = $2",
            &[&phone, &ctx.operator.id],
        )
        .await
        .ok()??;
    Some(row.get("public_key"))
}
