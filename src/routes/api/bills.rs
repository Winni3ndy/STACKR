use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct AirtimeRequest {
    pub phone: String,
    pub target_phone: String,
    pub product_type: String,
    pub amount_fiat: f64,
    pub pin: String,
}

#[derive(Debug, Deserialize)]
pub struct BillRequest {
    pub phone: String,
    pub bill_type: String,
    pub account_number: String,
    pub amount_fiat: f64,
    pub pin: String,
}

pub async fn buy_airtime(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<AirtimeRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);
    let phone = crate::utils::phone::normalize(&body.phone);
    let target = crate::utils::phone::normalize(&body.target_phone);

    // Verify PIN
    if !crate::services::ussd_menu::pin::verify_pin(&ctx, &phone, &body.pin).await {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid PIN"}));
    }

    match crate::services::bills::airbills::purchase_airtime(
        &ctx,
        &phone,
        &target,
        &body.product_type,
        body.amount_fiat,
    )
    .await
    {
        Ok(ref_id) => HttpResponse::Ok().json(serde_json::json!({
            "reference": ref_id,
            "status": "success",
        })),
        Err(e) => e.error_response(),
    }
}

pub async fn pay_bill(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<BillRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);
    let phone = crate::utils::phone::normalize(&body.phone);

    // Verify PIN
    if !crate::services::ussd_menu::pin::verify_pin(&ctx, &phone, &body.pin).await {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid PIN"}));
    }

    match crate::services::bills::airbills::pay_bill(
        &ctx,
        &phone,
        &body.bill_type,
        &body.account_number,
        body.amount_fiat,
    )
    .await
    {
        Ok(ref_id) => HttpResponse::Ok().json(serde_json::json!({
            "reference": ref_id,
            "status": "success",
        })),
        Err(e) => e.error_response(),
    }
}
