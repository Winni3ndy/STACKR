pub mod buy_airtime;
pub mod check_balance;
pub mod deposit;
pub mod limits;
pub mod main_menu;
pub mod pay_bills;
pub mod pay_merchant;
pub mod pin;
pub mod send_money;
pub mod swap;
pub mod view_wallet;
pub mod withdraw;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::context::RequestContext;
use crate::routes::ussd::UssdRequest;
use crate::AppState;

/// USSD session state stored in Redis with TTL.
/// Each menu branch stores its own data in `branch_data`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub phone: String,
    pub current_menu: MenuState,
    pub branch_data: serde_json::Value,
    pub pin_verified: bool,
    pub operator_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MenuState {
    MainMenu,
    // Send money flow
    SendMoneyEnterPhone,
    SendMoneyEnterAmount,
    SendMoneyConfirmPin,
    // Pay merchant flow
    PayMerchantEnterCode,
    PayMerchantEnterAmount,
    PayMerchantConfirmPin,
    // Buy airtime flow
    BuyAirtimeSelectType,
    BuyAirtimeEnterPhone,
    BuyAirtimeEnterAmount,
    BuyAirtimeConfirmPin,
    // Pay bills flow
    PayBillsSelectType,
    PayBillsEnterAccount,
    PayBillsEnterAmount,
    PayBillsConfirmPin,
    // Check balance
    CheckBalance,
    // View wallet
    ViewWallet,
    // Swap flow
    SwapSelectPair,
    SwapEnterAmount,
    SwapConfirmPin,
    // Withdraw (off-ramp) flow
    WithdrawSelectMethod,
    WithdrawEnterDetails,
    WithdrawEnterAmount,
    WithdrawConfirmPin,
    // Deposit (on-ramp) flow
    DepositSelectMethod,
    DepositEnterAmount,
    DepositConfirmPin,
    DepositShowInstructions,
    // Registration
    RegisterEnterPin,
    RegisterConfirmPin,
}

const SESSION_TTL_SECS: u64 = 300; // 5 minutes (USSD timeout is ~180s, buffer for processing)

/// Main entry point: process a USSD callback through the state machine.
pub async fn process_session(state: &AppState, req: &UssdRequest) -> String {
    // Resolve operator from service_code (shortcode)
    let ctx = match resolve_operator_context(state, &req.service_code).await {
        Ok(ctx) => ctx,
        Err(e) => {
            warn!(error = %e, shortcode = %req.service_code, "Failed to resolve operator");
            return "END Service temporarily unavailable. Please try again.".to_string();
        }
    };

    let session_key = format!("ussd_session:{}:{}", ctx.operator.id, req.session_id);

    // Load or create session
    let mut redis = state.redis.clone();
    let session: Option<SessionState> = redis
        .get::<_, Option<String>>(&session_key)
        .await
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());

    // Parse user input: Africa's Talking sends cumulative text separated by *
    // e.g., first input "1", second input "1*0712345678", third "1*0712345678*100"
    // We need only the latest segment
    let input = req.text.split('*').next_back().unwrap_or("").trim();

    let (session, response) = match session {
        Some(mut session) => {
            // Existing session — route to current menu handler
            let response = route_input(&ctx, &mut session, input).await;
            (session, response)
        }
        None => {
            // New session — check if user exists
            let db = match state.db_pool.get().await {
                Ok(c) => c,
                Err(e) => {
                    warn!(error = %e, "Database pool error");
                    return "END Service temporarily unavailable. Please try again.".to_string();
                }
            };

            let user_exists = db
                .query_opt(
                    "SELECT id FROM users WHERE phone = $1 AND operator_id = $2 AND is_active = TRUE",
                    &[&req.phone_number, &ctx.operator.id],
                )
                .await
                .ok()
                .flatten()
                .is_some();

            if user_exists {
                let session = SessionState {
                    phone: req.phone_number.clone(),
                    current_menu: MenuState::MainMenu,
                    branch_data: serde_json::json!({}),
                    pin_verified: false,
                    operator_id: ctx.operator.id,
                };
                let response = main_menu::render(&session);
                (session, response)
            } else {
                // New user — start registration
                let session = SessionState {
                    phone: req.phone_number.clone(),
                    current_menu: MenuState::RegisterEnterPin,
                    branch_data: serde_json::json!({}),
                    pin_verified: false,
                    operator_id: ctx.operator.id,
                };
                let response = "CON Welcome to Stackr!\n\
                    Your money, your phone.\n\n\
                    Create a 4-digit PIN to secure your account:"
                    .to_string();
                (session, response)
            }
        }
    };

    // Save session to Redis with TTL
    if let Ok(serialized) = serde_json::to_string(&session) {
        let _: Result<(), _> = redis
            .set_ex(&session_key, &serialized, SESSION_TTL_SECS)
            .await;
    }

    response
}

/// Resolve operator context from a USSD shortcode.
/// Falls back to the default operator if no match is found.
async fn resolve_operator_context(
    state: &AppState,
    shortcode: &str,
) -> Result<RequestContext, crate::error::AppError> {
    match RequestContext::from_shortcode(state, shortcode).await {
        Ok(ctx) => Ok(ctx),
        Err(_) => {
            // Fallback to default operator (for backward compat / single-tenant mode)
            RequestContext::from_default(state).await
        }
    }
}

/// Route user input to the appropriate menu handler based on current state
async fn route_input(ctx: &RequestContext, session: &mut SessionState, input: &str) -> String {
    match session.current_menu {
        MenuState::MainMenu => main_menu::handle_input(ctx, session, input).await,
        // Send money
        MenuState::SendMoneyEnterPhone => send_money::handle_enter_phone(ctx, session, input).await,
        MenuState::SendMoneyEnterAmount => {
            send_money::handle_enter_amount(ctx, session, input).await
        }
        MenuState::SendMoneyConfirmPin => send_money::handle_confirm_pin(ctx, session, input).await,
        // Pay merchant
        MenuState::PayMerchantEnterCode => {
            pay_merchant::handle_enter_code(ctx, session, input).await
        }
        MenuState::PayMerchantEnterAmount => {
            pay_merchant::handle_enter_amount(ctx, session, input).await
        }
        MenuState::PayMerchantConfirmPin => {
            pay_merchant::handle_confirm_pin(ctx, session, input).await
        }
        // Buy airtime
        MenuState::BuyAirtimeSelectType => {
            buy_airtime::handle_select_type(ctx, session, input).await
        }
        MenuState::BuyAirtimeEnterPhone => {
            buy_airtime::handle_enter_phone(ctx, session, input).await
        }
        MenuState::BuyAirtimeEnterAmount => {
            buy_airtime::handle_enter_amount(ctx, session, input).await
        }
        MenuState::BuyAirtimeConfirmPin => {
            buy_airtime::handle_confirm_pin(ctx, session, input).await
        }
        // Pay bills
        MenuState::PayBillsSelectType => pay_bills::handle_select_type(ctx, session, input).await,
        MenuState::PayBillsEnterAccount => {
            pay_bills::handle_enter_account(ctx, session, input).await
        }
        MenuState::PayBillsEnterAmount => pay_bills::handle_enter_amount(ctx, session, input).await,
        MenuState::PayBillsConfirmPin => pay_bills::handle_confirm_pin(ctx, session, input).await,
        // Balance
        MenuState::CheckBalance => check_balance::handle(ctx, session, input).await,
        // View Wallet
        MenuState::ViewWallet => view_wallet::handle(ctx, session).await,
        // Swap
        MenuState::SwapSelectPair => swap::handle_select_pair(ctx, session, input).await,
        MenuState::SwapEnterAmount => swap::handle_enter_amount(ctx, session, input).await,
        MenuState::SwapConfirmPin => swap::handle_confirm_pin(ctx, session, input).await,
        // Withdraw
        MenuState::WithdrawSelectMethod => {
            withdraw::handle_select_method(ctx, session, input).await
        }
        MenuState::WithdrawEnterDetails => {
            withdraw::handle_enter_details(ctx, session, input).await
        }
        MenuState::WithdrawEnterAmount => withdraw::handle_enter_amount(ctx, session, input).await,
        MenuState::WithdrawConfirmPin => withdraw::handle_confirm_pin(ctx, session, input).await,
        // Deposit
        MenuState::DepositSelectMethod => deposit::handle_select_method(ctx, session, input).await,
        MenuState::DepositEnterAmount => deposit::handle_enter_amount(ctx, session, input).await,
        MenuState::DepositConfirmPin => deposit::handle_confirm_pin(ctx, session, input).await,
        MenuState::DepositShowInstructions => {
            deposit::handle_show_instructions(ctx, session, input).await
        }
        // Registration
        MenuState::RegisterEnterPin => handle_register_enter_pin(ctx, session, input).await,
        MenuState::RegisterConfirmPin => handle_register_confirm_pin(ctx, session, input).await,
    }
}

async fn handle_register_enter_pin(
    _ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    if input.len() != 4 || !input.chars().all(|c| c.is_ascii_digit()) {
        return "CON PIN must be exactly 4 digits.\n\nEnter your PIN:".to_string();
    }

    session.branch_data["pin"] = serde_json::Value::String(input.to_string());
    session.current_menu = MenuState::RegisterConfirmPin;

    "CON Confirm your 4-digit PIN:".to_string()
}

async fn handle_register_confirm_pin(
    ctx: &RequestContext,
    session: &mut SessionState,
    input: &str,
) -> String {
    let original_pin = session.branch_data["pin"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if input != original_pin {
        session.current_menu = MenuState::RegisterEnterPin;
        session.branch_data = serde_json::json!({});
        return "CON PINs don't match. Let's try again.\n\nEnter a 4-digit PIN:".to_string();
    }

    // Create wallet and register user
    match crate::services::stellar::wallet::create_user_wallet(ctx, &session.phone, &original_pin)
        .await
    {
        Ok(public_key) => {
            info!(phone = %session.phone, "New user registered");
            session.current_menu = MenuState::MainMenu;
            session.pin_verified = true;
            session.branch_data = serde_json::json!({});

            format!(
                "CON Account created!\n\
                Your Stellar address:\n{}\n\n\
                {}",
                &public_key[..12],
                main_menu::menu_text()
            )
        }
        Err(e) => {
            warn!(error = %e, phone = %session.phone, "Failed to create wallet");
            "END Registration failed. Please try again later.".to_string()
        }
    }
}
