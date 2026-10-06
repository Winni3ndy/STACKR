use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use tracing::{info, instrument, warn};

use crate::middleware::{ip_allowlist, rate_limit, request_id};
use crate::services::ussd_menu;
use crate::AppState;

/// Africa's Talking USSD callback payload (x-www-form-urlencoded)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UssdRequest {
    pub session_id: String,
    pub phone_number: String,
    pub service_code: String,
    pub text: String,
}

/// USSD responses:
/// - Prefix with "CON " to keep session open (menu continues)
/// - Prefix with "END " to terminate session (final message)
#[instrument(skip(state, req), fields(session_id, phone))]
pub async fn ussd_callback(
    state: web::Data<AppState>,
    req: HttpRequest,
    form: web::Form<UssdRequest>,
) -> HttpResponse {
    let request_id = request_id::get_or_generate(&req);

    // IP allowlist enforcement
    if let Some(client_ip) = ip_allowlist::extract_client_ip(&req, &state.config) {
        if !ip_allowlist::is_at_ip_allowed(client_ip, &state.config) {
            warn!(
                ip = %client_ip,
                request_id = %request_id,
                "USSD callback from non-allowlisted IP"
            );
            return HttpResponse::Forbidden().body("END Access denied.");
        }

        // Rate limiting
        let mut redis = state.redis.clone();
        if rate_limit::check_rate_limit(
            &mut redis,
            client_ip,
            state.config.rate_limit_requests,
            state.config.rate_limit_window_secs,
        )
        .await
        .is_err()
        {
            return HttpResponse::TooManyRequests().body("END Too many requests. Try again later.");
        }
    }

    let ussd_req = form.into_inner();

    // Redact PIN segments from logs (4-digit sequences)
    let redacted_text = redact_pins(&ussd_req.text);
    info!(
        session_id = %ussd_req.session_id,
        phone = %ussd_req.phone_number,
        text = %redacted_text,
        request_id = %request_id,
        "USSD callback received"
    );

    // Process through USSD state machine
    let response = ussd_menu::process_session(&state, &ussd_req).await;

    info!(
        session_id = %ussd_req.session_id,
        response_prefix = &response[..3.min(response.len())],
        request_id = %request_id,
        "USSD response sent"
    );

    HttpResponse::Ok().content_type("text/plain").body(response)
}

/// Redact 4-digit PIN sequences in USSD text input for safe logging
fn redact_pins(text: &str) -> String {
    text.split('*')
        .map(|segment| {
            if segment.len() == 4 && segment.chars().all(|c| c.is_ascii_digit()) {
                "****"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("*")
}
