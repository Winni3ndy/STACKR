//! Anchor order management: tracks SEP-24 deposit/withdrawal lifecycle.
//! Follows Payce's PAJ order pattern: atomic state updates, event audit log,
//! fire-and-forget notifications.

use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use crate::context::RequestContext;
use crate::error::AppError;
use crate::AppState;

/// Anchor order status (mirrors Payce's PAJ pattern)
#[derive(Debug, Clone, PartialEq)]
pub enum AnchorOrderStatus {
    Pending,
    Completed,
    Error,
    Expired,
    Unknown,
}

impl AnchorOrderStatus {
    /// Parse status from anchor webhook/poll response.
    /// Handles multiple status field conventions across different anchors.
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "completed" | "success" | "succeeded" | "done" | "paid" => Self::Completed,
            "error" | "failed" | "failure" | "cancelled" | "canceled" | "rejected" => Self::Error,
            "expired" => Self::Expired,
            "pending"
            | "pending_user_transfer_start"
            | "pending_anchor"
            | "pending_stellar"
            | "pending_external"
            | "pending_trust"
            | "pending_user"
            | "pending_customer_info_update"
            | "processing"
            | "in_progress"
            | "incomplete" => Self::Pending,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Error => "error",
            Self::Expired => "expired",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_final(&self) -> bool {
        matches!(self, Self::Completed | Self::Error | Self::Expired)
    }
}

/// Create a new anchor order in the database.
#[allow(clippy::too_many_arguments)]
pub async fn create_order(
    ctx: &RequestContext,
    user_phone: &str,
    direction: &str,
    anchor_domain: &str,
    anchor_tx_id: &str,
    asset_code: &str,
    amount: Option<f64>,
    fiat_amount: Option<f64>,
    fiat_currency: Option<&str>,
    request_json: &Value,
    response_json: &Value,
) -> Result<Uuid, AppError> {
    let db = ctx.state.db_pool.get().await?;

    let order_id: Uuid = db
        .query_one(
            "INSERT INTO anchor_orders (user_id, direction, anchor_domain, anchor_tx_id,
                asset_code, amount, fiat_amount, fiat_currency, request_json, response_json, operator_id)
             SELECT u.id, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
             FROM users u WHERE u.phone = $1 AND u.operator_id = $11
             RETURNING id",
            &[
                &user_phone,
                &direction,
                &anchor_domain,
                &anchor_tx_id,
                &asset_code,
                &amount,
                &fiat_amount,
                &fiat_currency,
                &request_json,
                &response_json,
                &ctx.operator.id,
            ],
        )
        .await?
        .get("id");

    info!(
        order_id = %order_id,
        direction = %direction,
        anchor = %anchor_domain,
        phone = %user_phone,
        "Anchor order created"
    );

    Ok(order_id)
}

/// Atomically update an anchor order status.
/// Returns the user_id + direction if the update was applied (not already in final state).
/// Follows Payce's atomic webhook pattern: once completed/error/expired, status is immutable.
///
/// Note: This function takes &AppState (not &RequestContext) because webhooks don't have
/// operator context resolved yet — the order itself identifies the operator.
pub async fn update_order_status(
    state: &AppState,
    anchor_tx_id: &str,
    new_status: AnchorOrderStatus,
    payload: &Value,
) -> Result<Option<(Uuid, String, String)>, AppError> {
    let db = state.db_pool.get().await?;
    let status_str = new_status.as_str();

    // Atomic update: only update if not already in a final state
    let updated = db
        .query_opt(
            "UPDATE anchor_orders
             SET status = $2,
                 last_status_payload = $3,
                 updated_at = NOW()
             WHERE anchor_tx_id = $1
               AND status NOT IN ('completed', 'error', 'expired')
             RETURNING user_id, direction,
                       (SELECT phone FROM users WHERE id = user_id) as phone",
            &[&anchor_tx_id, &status_str, payload],
        )
        .await?;

    // Log the event regardless of whether the status was updated
    let _ = db
        .execute(
            "INSERT INTO anchor_order_events (order_id, status, payload)
             SELECT id, $2, $3 FROM anchor_orders WHERE anchor_tx_id = $1",
            &[&anchor_tx_id, &status_str, payload],
        )
        .await;

    match updated {
        Some(row) => {
            let user_id: Uuid = row.get("user_id");
            let direction: String = row.get("direction");
            let phone: String = row.get("phone");

            info!(
                anchor_tx_id = %anchor_tx_id,
                status = %status_str,
                direction = %direction,
                "Anchor order status updated"
            );

            // Fire-and-forget: update user activity
            let state_clone = state.clone();
            let phone_clone = phone.clone();
            let dir = direction.clone();
            let atid = anchor_tx_id.to_string();
            let payload_clone = payload.clone();
            tokio::spawn(async move {
                let _ = crate::services::activity::log(
                    &state_clone,
                    &phone_clone,
                    &atid,
                    &format!("anchor_{dir}_{status_str}"),
                    payload_clone,
                )
                .await;
            });

            // Fire-and-forget: SMS notification on final status
            // Note: For SMS we need operator context. Look up the operator_id from the order.
            if new_status.is_final() {
                let state_clone = state.clone();
                let phone_clone = phone.clone();
                let dir = direction.clone();
                let status = status_str.to_string();
                let atid = anchor_tx_id.to_string();
                tokio::spawn(async move {
                    // Resolve operator from the order
                    let operator_id = match get_order_operator_id(&state_clone, &atid).await {
                        Some(id) => id,
                        None => return,
                    };
                    let ctx = match crate::context::RequestContext::from_operator_id(
                        &state_clone,
                        operator_id,
                    )
                    .await
                    {
                        Ok(c) => c,
                        Err(_) => return,
                    };

                    let msg = match status.as_str() {
                        "completed" => format!("Stackr {} completed successfully!", dir),
                        "error" => format!("Stackr {} failed. Contact support.", dir),
                        "expired" => format!("Stackr {} expired. Please try again.", dir),
                        _ => return,
                    };
                    let _ = crate::services::sms::send_receipt(&ctx, &phone_clone, &msg).await;
                });
            }

            Ok(Some((user_id, direction, phone)))
        }
        None => {
            warn!(
                anchor_tx_id = %anchor_tx_id,
                status = %status_str,
                "Order already in final state — status not updated"
            );
            Ok(None)
        }
    }
}

/// Helper to get the operator_id for an anchor order (used by webhook handler)
async fn get_order_operator_id(state: &AppState, anchor_tx_id: &str) -> Option<Uuid> {
    let db = state.db_pool.get().await.ok()?;
    let row = db
        .query_opt(
            "SELECT operator_id FROM anchor_orders WHERE anchor_tx_id = $1",
            &[&anchor_tx_id],
        )
        .await
        .ok()??;
    row.get("operator_id")
}

/// Record the Stellar transaction hash for a withdrawal order
/// (after we send tokens to the anchor's address).
pub async fn set_stellar_tx_hash(
    state: &AppState,
    anchor_tx_id: &str,
    stellar_tx_hash: &str,
) -> Result<(), AppError> {
    let db = state.db_pool.get().await?;

    db.execute(
        "UPDATE anchor_orders SET stellar_tx_hash = $2, updated_at = NOW()
         WHERE anchor_tx_id = $1",
        &[&anchor_tx_id, &stellar_tx_hash],
    )
    .await?;

    Ok(())
}

/// Get the status of an anchor order by its anchor transaction ID.
pub async fn get_order_by_anchor_tx(
    state: &AppState,
    anchor_tx_id: &str,
) -> Result<Option<AnchorOrderInfo>, AppError> {
    let db = state.db_pool.get().await?;

    let row = db
        .query_opt(
            "SELECT ao.id, ao.status, ao.direction, ao.anchor_domain, ao.asset_code,
                    ao.amount, ao.fiat_amount, ao.fiat_currency, ao.stellar_tx_hash,
                    ao.operator_id, u.phone
             FROM anchor_orders ao
             JOIN users u ON u.id = ao.user_id
             WHERE ao.anchor_tx_id = $1",
            &[&anchor_tx_id],
        )
        .await?;

    Ok(row.map(|r| AnchorOrderInfo {
        id: r.get("id"),
        status: r.get("status"),
        direction: r.get("direction"),
        anchor_domain: r.get("anchor_domain"),
        asset_code: r.get("asset_code"),
        amount: r.get("amount"),
        fiat_amount: r.get("fiat_amount"),
        fiat_currency: r.get("fiat_currency"),
        stellar_tx_hash: r.get("stellar_tx_hash"),
        operator_id: r.get("operator_id"),
        phone: r.get("phone"),
    }))
}

#[derive(Debug)]
pub struct AnchorOrderInfo {
    pub id: Uuid,
    pub status: String,
    pub direction: String,
    pub anchor_domain: String,
    pub asset_code: String,
    pub amount: Option<f64>,
    pub fiat_amount: Option<f64>,
    pub fiat_currency: Option<String>,
    pub stellar_tx_hash: Option<String>,
    pub operator_id: Option<Uuid>,
    pub phone: String,
}
