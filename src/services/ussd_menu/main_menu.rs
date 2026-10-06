use super::{MenuState, SessionState};
use crate::context::RequestContext;

pub fn menu_text() -> &'static str {
    "1. Send Money\n\
     2. Pay Merchant\n\
     3. Buy Airtime/Data\n\
     4. Pay Bills\n\
     5. Check Balance\n\
     6. Swap Tokens\n\
     7. Withdraw to Bank\n\
     8. Deposit\n\
     9. My Wallet"
}

pub fn render(_session: &SessionState) -> String {
    format!("CON Welcome to Stackr\n\n{}", menu_text())
}

pub async fn handle_input(ctx: &RequestContext, session: &mut SessionState, input: &str) -> String {
    match input {
        "1" => {
            session.current_menu = MenuState::SendMoneyEnterPhone;
            "CON Enter recipient phone number:".to_string()
        }
        "2" => {
            session.current_menu = MenuState::PayMerchantEnterCode;
            "CON Enter merchant code:".to_string()
        }
        "3" => {
            session.current_menu = MenuState::BuyAirtimeSelectType;
            "CON Select:\n1. Airtime\n2. Data Bundle".to_string()
        }
        "4" => {
            session.current_menu = MenuState::PayBillsSelectType;
            "CON Select bill type:\n1. Electricity\n2. Cable TV\n3. Betting\n4. Internet"
                .to_string()
        }
        "5" => {
            session.current_menu = MenuState::CheckBalance;
            // CheckBalance handler will fetch and display
            super::check_balance::handle(ctx, session, "").await
        }
        "6" => {
            session.current_menu = MenuState::SwapSelectPair;
            "CON Select swap pair:\n1. USDC -> XLM\n2. XLM -> USDC\n3. USDC -> USDT".to_string()
        }
        "7" => {
            session.current_menu = MenuState::WithdrawSelectMethod;
            "CON Withdraw to:\n1. Bank Account\n2. Mobile Money".to_string()
        }
        "8" => {
            session.current_menu = MenuState::DepositSelectMethod;
            "CON Deposit from:\n1. Bank Transfer\n2. Mobile Money".to_string()
        }
        "9" => {
            session.current_menu = MenuState::ViewWallet;
            super::view_wallet::handle(ctx, session).await
        }
        _ => {
            format!("CON Invalid option.\n\n{}", menu_text())
        }
    }
}
