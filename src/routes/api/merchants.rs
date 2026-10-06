use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct MerchantPaymentRequest {
    pub sender_phone: String,
    pub merchant_code: String,
    pub amount: f64,
    pub pin: String,
}

pub async fn pay_merchant(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<MerchantPaymentRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);
    let phone = crate::utils::phone::normalize(&body.sender_phone);
    let code = body.merchant_code.trim().to_uppercase();

    if body.amount <= 0.0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Amount must be positive"}));
    }

    // Verify PIN
    if !crate::services::ussd_menu::pin::verify_pin(&ctx, &phone, &body.pin).await {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid PIN"}));
    }

    // Look up merchant
    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let merchant = match db
        .query_opt(
            "SELECT public_key, name FROM merchants WHERE merchant_code = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&code, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Merchant not found"}))
        }
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let merchant_key: String = merchant.get("public_key");

    match crate::services::stellar::transfer::send_usdc(&ctx, &phone, &merchant_key, body.amount)
        .await
    {
        Ok(tx_hash) => HttpResponse::Ok().json(serde_json::json!({
            "tx_hash": tx_hash,
            "status": "confirmed",
            "merchant_name": merchant.get::<_, String>("name"),
        })),
        Err(e) => e.error_response(),
    }
}
