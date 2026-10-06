use actix_web::{web, HttpRequest, HttpResponse};
use subtle::ConstantTimeEq;
use tracing::{info, warn};

use crate::services::anchor_orders;
use crate::AppState;

/// Webhook receiver for Stellar anchor callbacks (SEP-24 status updates).
///
/// Anchors POST status updates here when deposits/withdrawals change state.
/// Expected payload shape:
/// ```json
/// {
///     "id": "anchor-tx-id",
///     "status": "completed" | "error" | "expired" | ...,
///     "stellar_transaction_id": "optional-stellar-tx-hash",
///     "amount_in": "100.00",
///     "amount_out": "0.07",
///     ...
/// }
/// ```
pub async fn anchor_webhook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    // Verify webhook secret (constant-time comparison)
    let provided_secret = req
        .uri()
        .query()
        .and_then(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "k")
                .map(|(_, v)| v.to_string())
        })
        .unwrap_or_default();

    let expected = &state.config.webhook_secret;

    if !expected.is_empty()
        && provided_secret
            .as_bytes()
            .ct_eq(expected.as_bytes())
            .unwrap_u8()
            == 0
    {
        warn!("Webhook called with invalid secret");
        return HttpResponse::Unauthorized().finish();
    }

    // Parse the webhook payload
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "Invalid webhook payload");
            return HttpResponse::BadRequest().finish();
        }
    };

    let anchor_tx_id = match payload.get("id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            warn!("Webhook payload missing 'id' field");
            return HttpResponse::BadRequest().finish();
        }
    };

    let status_str = payload
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let new_status = anchor_orders::AnchorOrderStatus::parse(status_str);

    info!(
        anchor_tx = %anchor_tx_id,
        status = %status_str,
        "Anchor webhook received"
    );

    // Update the order status atomically
    match anchor_orders::update_order_status(&state, &anchor_tx_id, new_status.clone(), &payload)
        .await
    {
        Ok(Some((order_id, phone, direction))) => {
            info!(
                order_id = %order_id,
                phone = %phone,
                direction = %direction,
                new_status = %status_str,
                "Order status updated via webhook"
            );

            // If the anchor included a Stellar tx hash, record it
            if let Some(stellar_tx) = payload
                .get("stellar_transaction_id")
                .and_then(|v| v.as_str())
            {
                let _ = anchor_orders::set_stellar_tx_hash(&state, &anchor_tx_id, stellar_tx).await;
            }

            // SMS notification is handled inside anchor_orders::update_order_status
            // for terminal states (completed, error, expired)
        }
        Ok(None) => {
            // Order not found or status already final — idempotent, return 200
            info!(
                anchor_tx = %anchor_tx_id,
                "Webhook for unknown or already-final order (idempotent OK)"
            );
        }
        Err(e) => {
            warn!(error = %e, anchor_tx = %anchor_tx_id, "Failed to update order status");
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().finish()
}
