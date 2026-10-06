use redis::AsyncCommands;

use crate::error::AppError;
use crate::AppState;

const IDEMPOTENCY_TTL_SECS: u64 = 3600; // 1 hour

/// Check if a transaction with this idempotency key has already been processed.
/// Returns Some(tx_hash) if it exists, None if it's a new request.
pub async fn check(state: &AppState, key: &str) -> Result<Option<String>, AppError> {
    let mut redis = state.redis.clone();
    let result: Option<String> = redis.get(format!("idemp:{}", key)).await?;
    Ok(result)
}

/// Record a completed transaction's idempotency key → tx_hash mapping.
pub async fn record(state: &AppState, key: &str, tx_hash: &str) -> Result<(), AppError> {
    let mut redis = state.redis.clone();
    let redis_key = format!("idemp:{}", key);
    redis
        .set_ex::<_, _, ()>(&redis_key, tx_hash, IDEMPOTENCY_TTL_SECS)
        .await?;
    Ok(())
}
