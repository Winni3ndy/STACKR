use crate::context::RequestContext;

use super::SessionState;

pub async fn handle(ctx: &RequestContext, session: &SessionState) -> String {
    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return "END Service temporarily unavailable.".to_string(),
    };

    let user = match db
        .query_opt(
            "SELECT public_key, created_at FROM users WHERE phone = $1 AND operator_id = $2",
            &[&session.phone, &ctx.operator.id],
        )
        .await
    {
        Ok(Some(row)) => row,
        _ => return "END Account not found.".to_string(),
    };

    let public_key: String = user.get("public_key");

    // Break the wallet address into chunks for easier viewing on USSD
    let chunks: Vec<String> = public_key
        .as_bytes()
        .chunks(20)
        .map(|chunk| String::from_utf8_lossy(chunk).to_string())
        .collect();

    let mut display = String::from("END My Wallet\n\n");
    display.push_str(&format!("Phone: {}\n\n", session.phone));
    display.push_str("Stellar Address:\n");

    for chunk in chunks {
        display.push_str(&format!("{}\n", chunk));
    }

    display.push_str("\nShare this address to receive USDC from any Stellar wallet!");

    display
}
