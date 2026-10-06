use tracing::warn;

use crate::context::RequestContext;
use crate::services::stellar;

use super::SessionState;

pub async fn handle(ctx: &RequestContext, session: &mut SessionState, _input: &str) -> String {
    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return "END Service temporarily unavailable.".to_string(),
    };

    let user = match db
        .query_opt(
            "SELECT public_key FROM users WHERE phone = $1 AND operator_id = $2",
            &[&session.phone, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(row)) => row,
        _ => return "END Account not found.".to_string(),
    };

    let public_key: String = user.get("public_key");

    match stellar::client::get_balances(ctx, &public_key).await {
        Ok(balances) => {
            let mut display = String::from("END Your Balances:\n\n");
            for (asset, amount) in &balances {
                display.push_str(&format!("{}: {:.4}\n", asset, amount));
            }
            if balances.is_empty() {
                display.push_str("No assets found.\nDeposit to get started!");
            }
            display.push_str("\nThank you for using Stackr!");
            display
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch balances");
            "END Could not fetch balances. Try again later.".to_string()
        }
    }
}
