use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SwapRequest {
    pub phone: String,
    pub source_asset: String,
    pub dest_asset: String,
    pub amount: f64,
    pub pin: String,
}

pub async fn create_swap(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<SwapRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);
    let phone = crate::utils::phone::normalize(&body.phone);

    if body.amount <= 0.0 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Amount must be positive"}));
    }

    // Verify PIN
    if !crate::services::ussd_menu::pin::verify_pin(&ctx, &phone, &body.pin).await {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid PIN"}));
    }

    match crate::services::stellar::path_payment::swap(
        &ctx,
        &phone,
        &body.source_asset,
        &body.dest_asset,
        body.amount,
    )
    .await
    {
        Ok((tx_hash, received_amount)) => HttpResponse::Ok().json(serde_json::json!({
            "tx_hash": tx_hash,
            "status": "confirmed",
            "sent_amount": body.amount,
            "sent_asset": body.source_asset,
            "received_amount": received_amount,
            "received_asset": body.dest_asset,
        })),
        Err(e) => e.error_response(),
    }
}
