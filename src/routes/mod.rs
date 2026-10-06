pub mod api;
pub mod health;
pub mod ussd;
pub mod webhooks;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // API routes (must be registered before the catch-all scope)
    api::configure(cfg);

    // Core routes
    cfg.route("/health", web::get().to(health::health_check))
        .route("/ussd/callback", web::post().to(ussd::ussd_callback))
        .route("/webhooks/anchor", web::post().to(webhooks::anchor_webhook));
}
