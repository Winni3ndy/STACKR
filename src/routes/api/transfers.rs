use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub sender_phone: String,
    pub recipient_phone: String,
    pub amount: f64,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub tx_hash: String,
    pub status: String,
}

pub async fn create_transfer(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<TransferRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);

    let sender_phone = crate::utils::phone::normalize(&body.sender_phone);
    let recipient_phone = crate::utils::phone::normalize(&body.recipient_phone);

    if sender_phone.is_empty() || recipient_phone.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid phone number"}));
    }

    if body.amount <= 0.0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Amount must be positive"}));
    }

    // Verify PIN
    if !crate::services::ussd_menu::pin::verify_pin(&ctx, &sender_phone, &body.pin).await {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid PIN"}));
    }

    // Check daily limit
    if let Err(msg) =
        crate::services::ussd_menu::limits::check_daily_limit(&ctx, &sender_phone, body.amount)
            .await
    {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
    }

    // Look up recipient
    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let recipient = match db
        .query_opt(
            "SELECT public_key FROM users WHERE phone = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&recipient_phone, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Recipient not found"}))
        }
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let recipient_key: String = recipient.get("public_key");

    match crate::services::stellar::transfer::send_usdc(
        &ctx,
        &sender_phone,
        &recipient_key,
        body.amount,
    )
    .await
    {
        Ok(tx_hash) => {
            // Fire-and-forget daily spend update
            let ctx_clone = ctx.clone();
            let phone = sender_phone.clone();
            let amount = body.amount;
            tokio::spawn(async move {
                let _ = crate::services::ussd_menu::limits::update_daily_spend(
                    &ctx_clone, &phone, amount,
                )
                .await;
            });

            HttpResponse::Ok().json(TransferResponse {
                tx_hash,
                status: "confirmed".to_string(),
            })
        }
        Err(e) => e.error_response(),
    }
}
