use tracing::{info, warn};

use crate::context::RequestContext;
use crate::services::{anchor_orders, stellar};

use super::{MenuState, SessionState};

pub async fn handle_select_method(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    match input {
        "1" => {
            session.branch_data["method"] = serde_json::json!("bank");
            session.current_menu = MenuState::WithdrawEnterDetails;
            "CON Enter bank account number:".to_string()
        }
        "2" => {
            session.branch_data["method"] = serde_json::json!("mobile_money");
            session.current_menu = MenuState::WithdrawEnterDetails;
            "CON Enter mobile money number:".to_string()
        }
        "0" => {
            session.current_menu = MenuState::MainMenu;
            super::main_menu::render(session)
        }
        _ => "CON Invalid option.\n1. Bank Account\n2. Mobile Money\n0. Back".to_string(),
    }
}

pub async fn handle_enter_details(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if input.trim().is_empty() {
        return "CON Invalid input. Try again:".to_string();
    }

    session.branch_data["account_details"] = serde_json::json!(input.trim());
    session.current_menu = MenuState::WithdrawEnterAmount;

    "CON Enter amount to withdraw (USDC):".to_string()
}

pub async fn handle_enter_amount(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let amount: f64 = match input.parse() {
        Ok(a) if a > 0.0 => a,
        _ => return "CON Invalid amount. Enter a valid number:".to_string(),
    };

    let method = session.branch_data["method"]
        .as_str()
        .unwrap_or("bank")
        .to_string();
    let details = session.branch_data["account_details"]
        .as_str()
        .unwrap_or("N/A")
        .to_string();

    let currency = &ctx.operator.fiat_currency;

    // Get fiat equivalent for display
    let fiat_display =
        match crate::services::price_feed::usdc_to_fiat(&ctx.state, amount, currency).await {
            Ok(fiat) => format!(" (~{:.0} {})", fiat, currency),
            Err(_) => String::new(),
        };

    session.branch_data["amount"] = serde_json::json!(amount);
    session.current_menu = MenuState::WithdrawConfirmPin;

    format!(
        "CON Withdraw {:.2} USDC{} to {} ({})?\n\nEnter PIN to confirm:",
        amount, fiat_display, method, details
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
    let method = session.branch_data["method"]
        .as_str()
        .unwrap_or("bank")
        .to_string();
    let details = session.branch_data["account_details"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Check daily limit
    match super::limits::check_daily_limit(ctx, &session.phone, amount).await {
        Ok(()) => {}
        Err(msg) => return format!("END {}", msg),
    }

    // Initiate SEP-24 withdrawal via anchor
    let anchor_domain = ctx.operator.anchor_domain.clone();
    match stellar::anchors::sep24_withdraw(
        ctx,
        &anchor_domain,
        &session.phone,
        &ctx.operator.usdc_asset_code,
        amount,
    )
    .await
    {
        Ok(withdraw_response) => {
            let anchor_tx_id = withdraw_response.id.clone();

            // Record the order
            let _ = anchor_orders::create_order(
                ctx,
                &session.phone,
                "withdrawal",
                &anchor_domain,
                &anchor_tx_id,
                &ctx.operator.usdc_asset_code,
                Some(amount),
                None,
                Some(&ctx.operator.fiat_currency),
                &serde_json::json!({
                    "method": method,
                    "account_details": details,
                    "amount": amount,
                }),
                &serde_json::to_value(&withdraw_response).unwrap_or_default(),
            )
            .await;

            // If anchor provided a Stellar address + memo, send the tokens
            if let (Some(anchor_account), Some(_memo)) = (
                &withdraw_response.stellar_account_id,
                &withdraw_response.stellar_memo,
            ) {
                match stellar::transfer::send_usdc(ctx, &session.phone, anchor_account, amount)
                    .await
                {
                    Ok(tx_hash) => {
                        let _ =
                            anchor_orders::set_stellar_tx_hash(&ctx.state, &anchor_tx_id, &tx_hash)
                                .await;

                        info!(
                            phone = %session.phone,
                            anchor_tx = %anchor_tx_id,
                            stellar_tx = %tx_hash,
                            "Withdrawal: tokens sent to anchor"
                        );
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to send tokens to anchor");
                        return format!("END Withdrawal failed: {}", e.to_ussd_message());
                    }
                }
            }

            // Fire-and-forget activity log
            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let atid = anchor_tx_id.clone();
            let method_log = method.clone();
            let details_log = details.clone();
            let anchor_domain_clone = anchor_domain.clone();
            tokio::spawn(async move {
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &atid,
                    "withdrawal_initiated",
                    serde_json::json!({
                        "amount": amount,
                        "method": method_log,
                        "details": details_log,
                        "anchor": anchor_domain_clone,
                    }),
                )
                .await;
            });

            format!(
                "END Withdrawal of {:.2} USDC initiated.\n\
                 Method: {}\n\
                 Ref: {}\n\n\
                 You'll receive an SMS when complete.",
                amount,
                method,
                &anchor_tx_id[..8.min(anchor_tx_id.len())]
            )
        }
        Err(e) => {
            warn!(error = %e, phone = %session.phone, "Withdrawal initiation failed");
            format!("END {}", e.to_ussd_message())
        }
    }
}
