use tracing::warn;

use crate::AppState;

/// Log a user activity event. Called fire-and-forget via tokio::spawn
/// so USSD response latency is unaffected.
///
/// Uses INSERT ON CONFLICT (ref_id) DO UPDATE for race-safe upserts.
pub async fn log(
    state: &AppState,
    phone: &str,
    ref_id: &str,
    action: &str,
    detail: serde_json::Value,
) -> Result<(), ()> {
    let db = state.db_pool.get().await.map_err(|e| {
        warn!(error = %e, "DB pool error in activity log");
    })?;

    db.execute(
        "INSERT INTO user_activity (user_id, ref_id, action, detail)
         SELECT u.id, $2, $3, $4
         FROM users u WHERE u.phone = $1
         ON CONFLICT (ref_id) DO UPDATE SET
            action = EXCLUDED.action,
            detail = EXCLUDED.detail",
        &[&phone, &ref_id, &action, &detail],
    )
    .await
    .map_err(|e| {
        warn!(error = %e, ref_id = %ref_id, "Failed to log activity");
    })?;

    Ok(())
}
