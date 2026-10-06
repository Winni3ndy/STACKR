use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct WithdrawalRequest {
    pub phone: String,
    pub pin: String,
    pub amount: f64,
}

pub async fn create_withdrawal(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<WithdrawalRequest>,
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

    // Check daily limit
    if let Err(msg) =
        crate::services::ussd_menu::limits::check_daily_limit(&ctx, &phone, body.amount).await
    {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": msg}));
    }

    let anchor_domain = ctx.operator.anchor_domain.clone();
    match crate::services::stellar::anchors::sep24_withdraw(
        &ctx,
        &anchor_domain,
        &phone,
        &ctx.operator.usdc_asset_code,
        body.amount,
    )
    .await
    {
        Ok(withdraw_response) => {
            // Record the order
            let _ = crate::services::anchor_orders::create_order(
                &ctx,
                &phone,
                "withdrawal",
                &anchor_domain,
                &withdraw_response.id,
                &ctx.operator.usdc_asset_code,
                Some(body.amount),
                None,
                Some(&ctx.operator.fiat_currency),
                &serde_json::json!({"source": "api"}),
                &serde_json::to_value(&withdraw_response).unwrap_or_default(),
            )
            .await;

            // If anchor provided a Stellar address, send the tokens
            if let (Some(anchor_account), Some(_memo)) = (
                &withdraw_response.stellar_account_id,
                &withdraw_response.stellar_memo,
            ) {
                match crate::services::stellar::transfer::send_usdc(
                    &ctx,
                    &phone,
                    anchor_account,
                    body.amount,
                )
                .await
                {
                    Ok(tx_hash) => {
                        let _ = crate::services::anchor_orders::set_stellar_tx_hash(
                            &ctx.state,
                            &withdraw_response.id,
                            &tx_hash,
                        )
                        .await;
                    }
                    Err(e) => return e.error_response(),
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                "id": withdraw_response.id,
                "status": withdraw_response.status,
                "url": withdraw_response.url,
            }))
        }
        Err(e) => e.error_response(),
    }
}

pub async fn get_withdrawal(
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
            HttpResponse::NotFound().json(serde_json::json!({"error": "Withdrawal not found"}))
        }
        Err(e) => e.error_response(),
    }
}
