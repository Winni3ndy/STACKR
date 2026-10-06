use argon2::{Argon2, PasswordHash, PasswordVerifier};
use tracing::warn;

use crate::context::RequestContext;

/// Verify a user's PIN against the stored Argon2id hash.
/// Returns true if the PIN matches.
pub async fn verify_pin(ctx: &RequestContext, phone: &str, pin: &str) -> bool {
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "DB error during PIN verify");
            return false;
        }
    };

    let row = match db
        .query_opt(
            "SELECT pin_hash FROM users WHERE phone = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&phone, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(row)) => row,
        _ => return false,
    };

    let stored_hash: String = row.get("pin_hash");

    match PasswordHash::new(&stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(pin.as_bytes(), &parsed)
            .is_ok(),
        Err(e) => {
            warn!(error = %e, "Failed to parse stored PIN hash");
            false
        }
    }
}
