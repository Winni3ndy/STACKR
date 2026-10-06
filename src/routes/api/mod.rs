pub mod accounts;
pub mod bills;
pub mod deposits;
pub mod merchants;
pub mod rates;
pub mod swaps;
pub mod transactions;
pub mod transfers;
pub mod withdrawals;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Accounts
            .route("/accounts", web::post().to(accounts::create_account))
            .route("/accounts/{phone}", web::get().to(accounts::get_account))
            .route(
                "/accounts/{phone}/balance",
                web::get().to(accounts::get_balance),
            )
            // Transfers
            .route("/transfers", web::post().to(transfers::create_transfer))
            // Rates
            .route("/rates", web::get().to(rates::get_rates))
            // Transactions
            .route(
                "/transactions",
                web::get().to(transactions::list_transactions),
            )
            // Deposits
            .route("/deposits", web::post().to(deposits::create_deposit))
            .route("/deposits/{id}", web::get().to(deposits::get_deposit))
            // Withdrawals
            .route(
                "/withdrawals",
                web::post().to(withdrawals::create_withdrawal),
            )
            .route(
                "/withdrawals/{id}",
                web::get().to(withdrawals::get_withdrawal),
            )
            // Merchant payments
            .route(
                "/payments/merchant",
                web::post().to(merchants::pay_merchant),
            )
            // Bill payments
            .route("/payments/airtime", web::post().to(bills::buy_airtime))
            .route("/payments/bill", web::post().to(bills::pay_bill))
            // Swaps
            .route("/swaps", web::post().to(swaps::create_swap)),
    );
}
