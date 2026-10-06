use tracing::{info, warn};

use crate::context::RequestContext;

use super::{MenuState, SessionState};

pub async fn handle_select_type(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    match input {
        "1" => {
            session.branch_data["bill_type"] = serde_json::json!("airtime");
            session.current_menu = MenuState::BuyAirtimeEnterPhone;
            "CON Enter phone number for airtime:".to_string()
        }
        "2" => {
            session.branch_data["bill_type"] = serde_json::json!("data");
            session.current_menu = MenuState::BuyAirtimeEnterPhone;
            "CON Enter phone number for data:".to_string()
        }
        "0" => {
            session.current_menu = MenuState::MainMenu;
            super::main_menu::render(session)
        }
        _ => "CON Invalid option.\n1. Airtime\n2. Data Bundle\n0. Back".to_string(),
    }
}

pub async fn handle_enter_phone(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let phone = crate::utils::phone::normalize(input);

    if phone.is_empty() {
        return "CON Invalid phone number. Try again:".to_string();
    }

    session.branch_data["target_phone"] = serde_json::json!(phone);
    session.current_menu = MenuState::BuyAirtimeEnterAmount;

    let bill_type = session.branch_data["bill_type"]
        .as_str()
        .unwrap_or("airtime");
    let currency = &ctx.operator.fiat_currency;

    format!("CON Enter {} amount ({}):", bill_type, currency)
}

pub async fn handle_enter_amount(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let currency = &ctx.operator.fiat_currency;
    let amount: f64 = match input.parse() {
        Ok(a) if a >= 50.0 => a,
        Ok(_) => return format!("CON Minimum amount is 50 {}. Enter amount:", currency),
        _ => return "CON Invalid amount. Enter a valid number:".to_string(),
    };

    let bill_type = session.branch_data["bill_type"]
        .as_str()
        .unwrap_or("airtime")
        .to_string();
    let target_phone = session.branch_data["target_phone"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    session.branch_data["amount_fiat"] = serde_json::json!(amount);
    session.current_menu = MenuState::BuyAirtimeConfirmPin;

    format!(
        "CON Buy {:.0} {} {} for {}?\n\nEnter PIN to confirm:",
        amount, currency, bill_type, target_phone
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
    let target_phone = session.branch_data["target_phone"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let bill_type = session.branch_data["bill_type"]
        .as_str()
        .unwrap_or("airtime")
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

    // Debit USDC + call Airbills API
    match crate::services::bills::airbills::purchase_airtime(
        ctx,
        &session.phone,
        &target_phone,
        &bill_type,
        amount_fiat,
    )
    .await
    {
        Ok(ref_id) => {
            info!(
                phone = %session.phone,
                target = %target_phone,
                amount = amount_fiat,
                bill_type = %bill_type,
                ref_id = %ref_id,
                "Airtime/data purchase successful"
            );

            let ctx_clone = ctx.clone();
            let phone = session.phone.clone();
            let bt_spawn = bill_type.clone();
            let tp_spawn = target_phone.clone();
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
                        "type": bt_spawn,
                        "amount_fiat": amount_fiat,
                        "currency": currency_spawn,
                        "target": tp_spawn,
                    }),
                )
                .await;
            });

            format!(
                "END {} of {:.0} {} sent to {}\nRef: {}\n\nThank you!",
                bill_type, amount_fiat, currency, target_phone, ref_id
            )
        }
        Err(e) => {
            warn!(error = %e, "Airtime purchase failed");
            "END Purchase failed. Please try again later.".to_string()
        }
    }
}
