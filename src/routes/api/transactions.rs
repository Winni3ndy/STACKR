use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Deserialize;

use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub phone: String,
    pub limit: Option<i64>,
}

pub async fn list_transactions(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let phone = crate::utils::phone::normalize(&query.phone);
    let limit = query.limit.unwrap_or(20).min(100);

    let db = match state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => return crate::error::AppError::from(e).error_response(),
    };

    match db
        .query(
            "SELECT t.id, t.tx_type, t.status, t.amount, t.asset_code,
                    t.stellar_tx_hash, t.recipient_phone, t.merchant_code,
                    t.memo, t.created_at
             FROM transactions t
             JOIN users u ON u.id = t.user_id
             WHERE u.phone = $1 AND t.operator_id = $2
             ORDER BY t.created_at DESC
             LIMIT $3",
            &[&phone, &operator.id, &limit],
        )
        .await
    {
        Ok(rows) => {
            let txs: Vec<serde_json::Value> = rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.get::<_, uuid::Uuid>("id"),
                        "tx_type": r.get::<_, String>("tx_type"),
                        "status": r.get::<_, String>("status"),
                        "amount": r.get::<_, f64>("amount"),
                        "asset_code": r.get::<_, String>("asset_code"),
                        "stellar_tx_hash": r.get::<_, Option<String>>("stellar_tx_hash"),
                        "recipient_phone": r.get::<_, Option<String>>("recipient_phone"),
                        "merchant_code": r.get::<_, Option<String>>("merchant_code"),
                        "memo": r.get::<_, Option<String>>("memo"),
                        "created_at": r.get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
                    })
                })
                .collect();

            HttpResponse::Ok().json(serde_json::json!({ "transactions": txs }))
        }
        Err(e) => crate::error::AppError::from(e).error_response(),
    }
}
