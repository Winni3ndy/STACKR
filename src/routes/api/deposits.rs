use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    pub phone: String,
    pub pin: String,
    pub amount_fiat: Option<f64>,
}

pub async fn create_deposit(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<DepositRequest>,
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

    let usdc_amount = if let Some(fiat_amount) = body.amount_fiat {
        match crate::services::price_feed::fiat_to_usdc(
            &ctx.state,
            fiat_amount,
            &ctx.operator.fiat_currency,
        )
        .await
        {
            Ok(u) => Some(u),
            Err(e) => return e.error_response(),
        }
    } else {
        None
    };

    let anchor_domain = ctx.operator.anchor_domain.clone();
    match crate::services::stellar::anchors::sep24_deposit(
        &ctx,
        &anchor_domain,
        &phone,
        &ctx.operator.usdc_asset_code,
        usdc_amount,
    )
    .await
    {
        Ok(deposit_response) => {
            // Record the order
            let _ = crate::services::anchor_orders::create_order(
                &ctx,
                &phone,
                "deposit",
                &anchor_domain,
                &deposit_response.id,
                &ctx.operator.usdc_asset_code,
                usdc_amount,
                body.amount_fiat,
                Some(&ctx.operator.fiat_currency),
                &serde_json::json!({"source": "api"}),
                &serde_json::to_value(&deposit_response).unwrap_or_default(),
            )
            .await;

            HttpResponse::Ok().json(serde_json::json!({
                "id": deposit_response.id,
                "status": deposit_response.status,
                "url": deposit_response.url,
            }))
        }
        Err(e) => e.error_response(),
    }
}

pub async fn get_deposit(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let _operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let anchor_tx_id = path.into_inner();

    match crate::services::anchor_orders::get_order_by_anchor_tx(&state, &anchor_tx_id).await {
        Ok(Some(order)) => HttpResponse::Ok().json(serde_json::json!({
            "id": order.id,
            "status": order.status,
            "direction": order.direction,
            "amount": order.amount,
            "fiat_amount": order.fiat_amount,
            "fiat_currency": order.fiat_currency,
            "stellar_tx_hash": order.stellar_tx_hash,
        })),
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Deposit not found"}))
        }
        Err(e) => e.error_response(),
    }
}
