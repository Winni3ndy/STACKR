use std::sync::Arc;

use actix_web::{web, HttpRequest};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::operator::OperatorConfig;
use crate::AppState;

/// Extract and validate an API key from the Authorization header.
/// Returns the resolved OperatorConfig for the authenticated operator.
///
/// Expected header format: `Authorization: Bearer sk_...`
pub async fn authenticate(
    state: &web::Data<AppState>,
    req: &HttpRequest,
) -> Result<Arc<OperatorConfig>, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let api_key = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format".to_string()))?
        .trim();

    if api_key.is_empty() {
        return Err(AppError::Unauthorized("Empty API key".to_string()));
    }

    // SHA-256 hash the key for lookup
    let key_hash = hex::encode(Sha256::digest(api_key.as_bytes()));

    let db = state.db_pool.get().await?;

    let row = db
        .query_opt(
            "SELECT ak.operator_id
             FROM api_keys ak
             JOIN operators o ON o.id = ak.operator_id
             WHERE ak.key_hash = $1
               AND ak.is_active = TRUE
               AND o.is_active = TRUE",
            &[&key_hash],
        )
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

    let operator_id: uuid::Uuid = row.get("operator_id");

    // Update last_used_at (fire-and-forget)
    let state_clone = state.get_ref().clone();
    let key_hash_clone = key_hash.clone();
    tokio::spawn(async move {
        if let Ok(db) = state_clone.db_pool.get().await {
            let _ = db
                .execute(
                    "UPDATE api_keys SET last_used_at = NOW() WHERE key_hash = $1",
                    &[&key_hash_clone],
                )
                .await;
        }
    });

    // Load operator config (cached)
    crate::operator::load_by_id(state.get_ref(), operator_id).await
}
