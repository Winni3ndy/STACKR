use tracing::{info, warn};

use crate::context::RequestContext;
use crate::services::stellar;

use super::{MenuState, SessionState};

pub async fn handle_enter_code(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if input == "0" {
        session.current_menu = MenuState::MainMenu;
        return super::main_menu::render(session);
    }

    let code = input.trim().to_uppercase();

    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return "END Service temporarily unavailable.".to_string(),
    };

    let merchant = db
        .query_opt(
            "SELECT name, public_key FROM merchants WHERE merchant_code = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&code, &ctx.operator.id],
        )
        .await
        .ok()
        .flatten();

    match merchant {
        Some(row) => {
            let name: String = row.get("name");
            let public_key: String = row.get("public_key");
            session.branch_data["merchant_code"] = serde_json::json!(code);
            session.branch_data["merchant_name"] = serde_json::json!(name);
            session.branch_data["merchant_key"] = serde_json::json!(public_key);
            session.current_menu = MenuState::PayMerchantEnterAmount;
            format!("CON Pay to: {}\n\nEnter amount (USDC):", name)
        }
        None => "CON Merchant not found.\n\nEnter merchant code or 0 for menu:".to_string(),
    }
}

pub async fn handle_enter_amount(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if input == "0" {
        session.current_menu = MenuState::MainMenu;
        return super::main_menu::render(session);
    }

    let amount: f64 = match input.parse() {
        Ok(a) if a > 0.0 => a,
        _ => return "CON Invalid amount. Enter a valid number:".to_string(),
    };

    let merchant_name = session.branch_data["merchant_name"]
        .as_str()
        .unwrap_or("Merchant")
        .to_string();

    session.branch_data["amount"] = serde_json::json!(amount);
    session.current_menu = MenuState::PayMerchantConfirmPin;

    format!(
        "CON Pay {:.2} USDC to {}?\n\nEnter PIN to confirm:",
        amount, merchant_name
    )
}

pub async fn handle_confirm_pin(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if !super::pin::verify_pin(ctx, &session.phone, input).await {
        return "CON Incorrect PIN. Try again:".to_string();
    }

    let amount = session.branch_data["amount"].as_f64().unwrap_or(0.0);
    let merchant_key = session.branch_data["merchant_key"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let merchant_name = session.branch_data["merchant_name"]
        .as_str()
        .unwrap_or("Merchant")
        .to_string();
    let merchant_code = session.branch_data["merchant_code"]
        .as_str()
        .unwrap_or("")
        .to_string();

    match stellar::transfer::send_usdc(ctx, &session.phone, &merchant_key, amount).await {
        Ok(tx_hash) => {
            info!(
                from = %session.phone,
                merchant = %merchant_code,
                amount = amount,
                tx_hash = %tx_hash,
                "Merchant payment successful"
            );

            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let mc = merchant_code.clone();
            let mn_spawn = merchant_name.clone();
            let tx_hash_spawn = tx_hash.clone();
            tokio::spawn(async move {
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &tx_hash_spawn,
                    "merchant_payment",
                    serde_json::json!({
                        "amount": amount,
                        "merchant_code": mc,
                        "merchant_name": mn_spawn,
                        "asset": "USDC"
                    }),
                )
                .await;
            });

            format!(
                "END Paid {:.2} USDC to {}\nTx: {}...\n\nThank you!",
                amount,
                merchant_name,
                &tx_hash[..12.min(tx_hash.len())]
            )
        }
        Err(e) => {
            warn!(error = %e, "Merchant payment failed");
            format!("END {}", e.to_ussd_message())
        }
    }
}
