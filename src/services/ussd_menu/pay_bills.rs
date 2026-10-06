use tracing::{info, warn};

use crate::context::RequestContext;

use super::{MenuState, SessionState};

pub async fn handle_select_type(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let bill_type = match input {
        "1" => "electricity",
        "2" => "cable_tv",
        "3" => "betting",
        "4" => "internet",
        "0" => {
            session.current_menu = MenuState::MainMenu;
            return super::main_menu::render(session);
        }
        _ => {
            return "CON Invalid option.\n1. Electricity\n2. Cable TV\n3. Betting\n4. Internet\n0. Back"
                .to_string();
        }
    };

    session.branch_data["bill_type"] = serde_json::json!(bill_type);
    session.current_menu = MenuState::PayBillsEnterAccount;

    let prompt = match bill_type {
        "electricity" => "CON Enter meter number:",
        "cable_tv" => "CON Enter smart card number:",
        "betting" => "CON Enter account ID:",
        "internet" => "CON Enter customer ID:",
        _ => "CON Enter account number:",
    };

    prompt.to_string()
}

pub async fn handle_enter_account(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if input.trim().is_empty() {
        return "CON Invalid account number. Try again:".to_string();
    }

    session.branch_data["account_number"] = serde_json::json!(input.trim());
    session.current_menu = MenuState::PayBillsEnterAmount;

    format!("CON Enter amount ({}):", ctx.operator.fiat_currency)
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

    let bill_type = session.branch_data["bill_type"]
        .as_str()
        .unwrap_or("bill")
        .to_string();
    let account = session.branch_data["account_number"]
        .as_str()
        .unwrap_or("N/A")
        .to_string();
    let currency = &ctx.operator.fiat_currency;

    session.branch_data["amount_fiat"] = serde_json::json!(amount);
    session.current_menu = MenuState::PayBillsConfirmPin;

    format!(
        "CON Pay {:.0} {} {} for account {}?\n\nEnter PIN to confirm:",
        amount, currency, bill_type, account
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

    let amount_fiat = session.branch_data["amount_fiat"].as_f64().unwrap_or(0.0);
    let bill_type = session.branch_data["bill_type"]
        .as_str()
        .unwrap_or("bill")
        .to_string();
    let account = session.branch_data["account_number"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let currency = ctx.operator.fiat_currency.clone();

    // Convert fiat to USDC for daily limit check
    let usdc_equiv =
        match crate::services::price_feed::fiat_to_usdc(&ctx.state, amount_fiat, &currency).await {
            Ok(u) => u,
            Err(_) => return "END Could not get exchange rate. Try again later.".to_string(),
        };

    // Check daily limit (in USDC)
    match super::limits::check_daily_limit(ctx, &session.phone, usdc_equiv).await {
        Ok(()) => {}
        Err(msg) => return format!("END {}", msg),
    }

    // Debit USDC + call Airbills bill payment API
    match crate::services::bills::airbills::pay_bill(
        ctx,
        &session.phone,
        &bill_type,
        &account,
        amount_fiat,
    )
    .await
    {
        Ok(ref_id) => {
            info!(
                phone = %session.phone,
                bill_type = %bill_type,
                account = %account,
                amount = amount_fiat,
                "Bill payment successful"
            );

            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let bt = bill_type.clone();
            let acct_spawn = account.clone();
            let ref_id_spawn = ref_id.clone();
            let currency_spawn = currency.clone();
            tokio::spawn(async move {
                let _ = super::limits::update_daily_spend(&ctx_clone, &phone, usdc_equiv).await;
                let _ = crate::services::activity::log(
                    &ctx_clone.state,
                    &phone,
                    &ref_id_spawn,
                    "bill_payment",
                    serde_json::json!({
                        "type": bt,
                        "account": acct_spawn,
                        "amount_fiat": amount_fiat,
                        "currency": currency_spawn,
                    }),
                )
                .await;
            });

            format!(
                "END {} bill paid: {:.0} {}\nAccount: {}\nRef: {}\n\nThank you!",
                bill_type, amount_fiat, currency, account, ref_id
            )
        }
        Err(e) => {
            warn!(error = %e, "Bill payment failed");
            "END Payment failed. Please try again later.".to_string()
        }
    }
}
