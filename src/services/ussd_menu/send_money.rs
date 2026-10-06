use tracing::{info, warn};

use crate::context::RequestContext;
use crate::services::stellar;

use super::{MenuState, SessionState};

pub async fn handle_enter_phone(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let phone = crate::utils::phone::normalize(input);

    if phone.is_empty() {
        return "CON Invalid phone number. Try again:".to_string();
    }

    // Check if recipient exists
    let db = match ctx.state.db_pool.get().await {
        Ok(c) => c,
        Err(_) => return "END Service temporarily unavailable.".to_string(),
    };

    let recipient = db
        .query_opt(
            "SELECT public_key FROM users WHERE phone = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&phone, &ctx.operator.id],
        )
        .await
        .ok()
        .flatten();

    match recipient {
        Some(row) => {
            let recipient_key: String = row.get("public_key");
            session.branch_data["recipient_phone"] = serde_json::json!(phone);
            session.branch_data["recipient_key"] = serde_json::json!(recipient_key);
            session.current_menu = MenuState::SendMoneyEnterAmount;
            format!(
                "CON Sending to {}\n\nEnter amount (USDC):",
                mask_phone(&phone)
            )
        }
        None => "CON Recipient not found on Stackr.\n\n\
             Enter another phone number or 0 for main menu:"
            .to_string(),
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

    let recipient_phone = session.branch_data["recipient_phone"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    session.branch_data["amount"] = serde_json::json!(amount);
    session.current_menu = MenuState::SendMoneyConfirmPin;

    format!(
        "CON Send {:.2} USDC to {}?\n\nEnter PIN to confirm:",
        amount,
        mask_phone(&recipient_phone)
    )
}

pub async fn handle_confirm_pin(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    // Verify PIN
    if !super::pin::verify_pin(ctx, &session.phone, input).await {
        return "CON Incorrect PIN. Try again:".to_string();
    }

    let amount = session.branch_data["amount"].as_f64().unwrap_or(0.0);
    let recipient_key = session.branch_data["recipient_key"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let recipient_phone = session.branch_data["recipient_phone"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Check daily limit
    match super::limits::check_daily_limit(ctx, &session.phone, amount).await {
        Ok(()) => {}
        Err(msg) => return format!("END {}", msg),
    }

    // Execute transfer
    match stellar::transfer::send_usdc(ctx, &session.phone, &recipient_key, amount).await {
        Ok(tx_hash) => {
            info!(
                from = %session.phone,
                to = %recipient_phone,
                amount = amount,
                tx_hash = %tx_hash,
                "Transfer successful"
            );

            let tx_hash_display = tx_hash.clone();

            // Fire-and-forget: update daily spend + activity log
            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let tx_hash_activity = tx_hash.clone();
            let rp_activity = recipient_phone.clone();
            tokio::spawn(async move {
                let _ = super::limits::update_daily_spend(&ctx_clone, &phone, amount).await;
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &tx_hash_activity,
                    "transfer_sent",
                    serde_json::json!({
                        "amount": amount,
                        "recipient": rp_activity,
                        "asset": "USDC"
                    }),
                )
                .await;
            });

            // SMS receipt (fire-and-forget)
            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let rp = recipient_phone.clone();
            let tx_hash_sms = tx_hash;
            tokio::spawn(async move {
                let _ = crate::services::sms::send_receipt(
                    &ctx_clone,
                    &phone,
                    &format!(
                        "Sent {:.2} USDC to {}. Tx: {}",
                        amount,
                        rp,
                        &tx_hash_sms[..8.min(tx_hash_sms.len())]
                    ),
                )
                .await;
            });

            format!(
                "END Sent {:.2} USDC to {}\nTx: {}...\n\nThank you for using Stackr!",
                amount,
                mask_phone(&recipient_phone),
                &tx_hash_display[..12.min(tx_hash_display.len())]
            )
        }
        Err(e) => {
            warn!(error = %e, phone = %session.phone, "Transfer failed");
            format!("END {}", e.to_ussd_message())
        }
    }
}

fn mask_phone(phone: &str) -> String {
    if phone.len() > 6 {
        format!("{}***{}", &phone[..4], &phone[phone.len() - 3..])
    } else {
        "***".to_string()
    }
}
