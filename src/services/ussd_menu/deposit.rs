use tracing::{info, warn};

use crate::context::RequestContext;
use crate::services::{anchor_orders, stellar};

use super::{MenuState, SessionState};

pub async fn handle_select_method(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    match input {
        "1" => {
            session.branch_data["method"] = serde_json::json!("bank");
            session.current_menu = MenuState::DepositEnterAmount;
            format!(
                "CON Enter amount to deposit ({}):",
                ctx.operator.fiat_currency
            )
        }
        "2" => {
            session.branch_data["method"] = serde_json::json!("mobile_money");
            session.current_menu = MenuState::DepositEnterAmount;
            format!(
                "CON Enter amount to deposit ({}):",
                ctx.operator.fiat_currency
            )
        }
        "0" => {
            session.current_menu = MenuState::MainMenu;
            super::main_menu::render(session)
        }
        _ => "CON Invalid option.\n1. Bank Transfer\n2. Mobile Money\n0. Back".to_string(),
    }
}

pub async fn handle_enter_amount(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let fiat_amount: f64 = match input.parse() {
        Ok(a) if a > 0.0 => a,
        _ => return "CON Invalid amount. Enter a valid number:".to_string(),
    };

    let currency = &ctx.operator.fiat_currency;

    // Convert fiat to USDC for display
    let usdc_display =
        match crate::services::price_feed::fiat_to_usdc(&ctx.state, fiat_amount, currency).await {
            Ok(usdc) => format!(" (~{:.2} USDC)", usdc),
            Err(_) => String::new(),
        };

    let method = session.branch_data["method"]
        .as_str()
        .unwrap_or("bank")
        .to_string();

    session.branch_data["fiat_amount"] = serde_json::json!(fiat_amount);
    session.current_menu = MenuState::DepositConfirmPin;

    format!(
        "CON Deposit {:.0} {}{} via {}?\n\nEnter PIN to confirm:",
        fiat_amount, currency, usdc_display, method
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

    let fiat_amount = session.branch_data["fiat_amount"].as_f64().unwrap_or(0.0);
    let method = session.branch_data["method"]
        .as_str()
        .unwrap_or("bank")
        .to_string();

    let currency = ctx.operator.fiat_currency.clone();

    // Convert to USDC equivalent for anchor request
    let usdc_amount =
        match crate::services::price_feed::fiat_to_usdc(&ctx.state, fiat_amount, &currency).await {
            Ok(a) => a,
            Err(e) => {
                warn!(error = %e, "Failed to get exchange rate for deposit");
                return "END Could not get exchange rate. Please try again later.".to_string();
            }
        };

    // Initiate SEP-24 deposit via anchor
    let anchor_domain = ctx.operator.anchor_domain.clone();
    match stellar::anchors::sep24_deposit(
        ctx,
        &anchor_domain,
        &session.phone,
        &ctx.operator.usdc_asset_code,
        Some(usdc_amount),
    )
    .await
    {
        Ok(deposit_response) => {
            let anchor_tx_id = deposit_response.id.clone();

            // Record the order
            let _ = anchor_orders::create_order(
                ctx,
                &session.phone,
                "deposit",
                &anchor_domain,
                &anchor_tx_id,
                &ctx.operator.usdc_asset_code,
                Some(usdc_amount),
                Some(fiat_amount),
                Some(&currency),
                &serde_json::json!({
                    "method": method,
                    "fiat_amount": fiat_amount,
                    "usdc_amount": usdc_amount,
                }),
                &serde_json::to_value(&deposit_response).unwrap_or_default(),
            )
            .await;

            // Extract payment instructions from anchor response
            let instructions = if let Some(extra_info) = &deposit_response.extra_info {
                extra_info.clone()
            } else {
                match method.as_str() {
                    "bank" => "Transfer to the bank account details sent via SMS".to_string(),
                    _ => "Send to the mobile money number sent via SMS".to_string(),
                }
            };

            // Fire-and-forget activity log
            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let atid = anchor_tx_id.clone();
            let method_clone = method.clone();
            let anchor_domain_clone = anchor_domain.clone();
            tokio::spawn(async move {
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &atid,
                    "deposit_initiated",
                    serde_json::json!({
                        "fiat_amount": fiat_amount,
                        "usdc_amount": usdc_amount,
                        "method": method_clone,
                        "anchor": anchor_domain_clone,
                    }),
                )
                .await;
            });

            info!(
                phone = %session.phone,
                anchor_tx = %anchor_tx_id,
                fiat = fiat_amount,
                usdc = usdc_amount,
                "Deposit initiated via anchor"
            );

            format!(
                "END Deposit of {:.0} {} initiated.\n\
                 Method: {}\n\
                 Ref: {}\n\n\
                 {}\n\n\
                 SMS confirmation will be sent\nwhen USDC is credited.",
                fiat_amount,
                currency,
                method,
                &anchor_tx_id[..8.min(anchor_tx_id.len())],
                instructions
            )
        }
        Err(e) => {
            warn!(error = %e, phone = %session.phone, "Deposit initiation failed");
            format!("END {}", e.to_ussd_message())
        }
    }
}

pub async fn handle_show_instructions(
    _ctx: &RequestContext,
    session: &mut SessionState,
    _input: &str,
) -> String {
    session.current_menu = MenuState::MainMenu;
    super::main_menu::render(session)
}
