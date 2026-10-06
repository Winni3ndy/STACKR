use tracing::{info, warn};

use crate::context::RequestContext;
use crate::error::AppError;

/// Send an SMS receipt to a user via Africa's Talking SMS API.
pub async fn send_receipt(
    ctx: &RequestContext,
    phone: &str,
    message: &str,
) -> Result<(), AppError> {
    let url = "https://api.africastalking.com/version1/messaging";

    let response = ctx
        .state
        .http_client
        .post(url)
        .header("apiKey", &ctx.operator.at_api_key)
        .header("Accept", "application/json")
        .form(&[
            ("username", ctx.operator.at_username.as_str()),
            ("to", phone),
            ("message", message),
            ("from", ctx.operator.at_sender_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: "AfricasTalking SMS".to_string(),
            detail: e.to_string(),
        })?;

    if response.status().is_success() {
        info!(phone = %phone, "SMS receipt sent");
        Ok(())
    } else {
        let body = response.text().await.unwrap_or_default();
        warn!(phone = %phone, body = %body, "SMS send failed");
        Err(AppError::ExternalApiError {
            service: "AfricasTalking SMS".to_string(),
            detail: body,
        })
    }
}
