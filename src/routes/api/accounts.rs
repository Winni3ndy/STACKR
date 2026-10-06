use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

use crate::context::RequestContext;
use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub phone: String,
    pub pin: String,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: uuid::Uuid,
    pub phone: String,
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct BalanceEntry {
    pub asset: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balances: Vec<BalanceEntry>,
}

pub async fn create_account(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<CreateAccountRequest>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);

    let phone = crate::utils::phone::normalize(&body.phone);
    if phone.is_empty() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "Invalid phone number"}));
    }

    if body.pin.len() != 4 || !body.pin.chars().all(|c| c.is_ascii_digit()) {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"error": "PIN must be exactly 4 digits"}));
    }

    match crate::services::stellar::wallet::create_user_wallet(&ctx, &phone, &body.pin).await {
        Ok(public_key) => {
            // Fetch the user ID
            let db = match ctx.state.db_pool.get().await {
                Ok(c) => c,
                Err(e) => return crate::error::AppError::from(e).error_response(),
            };

            let id = db
                .query_opt(
                    "SELECT id FROM users WHERE phone = $1 AND operator_id = $2",
                    &[&phone, &ctx.operator.id],
                )
                .await
                .ok()
                .flatten()
                .map(|r| r.get::<_, uuid::Uuid>("id"))
                .unwrap_or_else(uuid::Uuid::new_v4);

            HttpResponse::Created().json(AccountResponse {
                id,
                phone,
                public_key,
            })
        }
        Err(e) => e.error_response(),
    }
}

pub async fn get_account(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let phone = crate::utils::phone::normalize(&path.into_inner());

    let db = match state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    match db
        .query_opt(
            "SELECT id, phone, public_key, kyc_tier, is_active, created_at
             FROM users WHERE phone = $1 AND operator_id = $2",
            &[&phone, &operator.id],
        )
        .await
    {
        Ok(Some(row)) => {
            let resp = serde_json::json!({
                "id": row.get::<_, uuid::Uuid>("id"),
                "phone": row.get::<_, String>("phone"),
                "public_key": row.get::<_, String>("public_key"),
                "kyc_tier": row.get::<_, i16>("kyc_tier"),
                "is_active": row.get::<_, bool>("is_active"),
                "created_at": row.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
            });
            HttpResponse::Ok().json(resp)
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Account not found"}))
        }
        Err(e) => crate::error::AppError::from(e).error_response(),
    }
}

pub async fn get_balance(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let ctx = RequestContext::new(state.get_ref().clone(), operator);
    let phone = crate::utils::phone::normalize(&path.into_inner());

    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let row = match db
        .query_opt(
            "SELECT public_key FROM users WHERE phone = $1 AND operator_id = $2",
            &[&phone, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Account not found"}))
        }
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    let public_key: String = row.get("public_key");

    match crate::services::stellar::client::get_balances(&ctx, &public_key).await {
        Ok(balances) => {
            let entries: Vec<BalanceEntry> = balances
                .into_iter()
                .map(|(asset, amount)| BalanceEntry { asset, amount })
                .collect();
            HttpResponse::Ok().json(BalanceResponse { balances: entries })
        }
        Err(e) => e.error_response(),
    }
}
