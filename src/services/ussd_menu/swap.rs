use tracing::{info, warn};

use crate::context::RequestContext;
use crate::services::stellar;

use super::{MenuState, SessionState};

pub async fn handle_select_pair(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let (source_asset, dest_asset) = match input {
        "1" => ("USDC", "XLM"),
        "2" => ("XLM", "USDC"),
        "3" => ("USDC", "USDT"),
        "0" => {
            session.current_menu = MenuState::MainMenu;
            return super::main_menu::render(session);
        }
        _ => {
            return "CON Invalid option.\n1. USDC -> XLM\n2. XLM -> USDC\n3. USDC -> USDT\n0. Back"
                .to_string();
        }
    };

    let prompt = format!(
        "CON Swap {} -> {}\n\nEnter amount of {} to swap:",
        source_asset, dest_asset, source_asset
    );
    session.branch_data["source_asset"] = serde_json::json!(source_asset);
    session.branch_data["dest_asset"] = serde_json::json!(dest_asset);
    session.current_menu = MenuState::SwapEnterAmount;

    prompt
}

pub async fn handle_enter_amount(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let amount: f64 = match input.parse() {
        Ok(a) if a > 0.0 => a,
        _ => return "CON Invalid amount. Enter a valid number:".to_string(),
    };

    let source = session.branch_data["source_asset"]
        .as_str()
        .unwrap_or("?")
        .to_string();
    let dest = session.branch_data["dest_asset"]
        .as_str()
        .unwrap_or("?")
        .to_string();

    session.branch_data["amount"] = serde_json::json!(amount);
    session.current_menu = MenuState::SwapConfirmPin;

    format!(
        "CON Swap {:.4} {} -> {}?\n\nEnter PIN to confirm:",
        amount, source, dest
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
    let source_asset = session.branch_data["source_asset"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let dest_asset = session.branch_data["dest_asset"]
        .as_str()
        .unwrap_or("")
        .to_string();

    match stellar::path_payment::swap(ctx, &session.phone, &source_asset, &dest_asset, amount).await
    {
        Ok((tx_hash, received_amount)) => {
            info!(
                phone = %session.phone,
                source = %source_asset,
                dest = %dest_asset,
                sent = amount,
                received = received_amount,
                tx_hash = %tx_hash,
                "Swap successful"
            );

            let tx_hash_display = tx_hash.clone();
            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let sa = source_asset.clone();
            let da = dest_asset.clone();
            tokio::spawn(async move {
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &tx_hash,
                    "swap",
                    serde_json::json!({
                        "source_asset": sa,
                        "dest_asset": da,
                        "sent": amount,
                        "received": received_amount,
                    }),
                )
                .await;
            });

            format!(
                "END Swapped {:.4} {} -> {:.4} {}\nTx: {}...\n\nThank you!",
                amount,
                source_asset,
                received_amount,
                dest_asset,
                &tx_hash_display[..12.min(tx_hash_display.len())]
            )
        }
        Err(e) => {
            warn!(error = %e, "Swap failed");
            format!("END {}", e.to_ussd_message())
        }
    }
}
