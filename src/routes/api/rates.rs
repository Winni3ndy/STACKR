use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use serde::Serialize;

use crate::middleware::api_auth;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct RatesResponse {
    pub usdc_to_fiat: f64,
    pub fiat_currency: String,
}

pub async fn get_rates(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let operator = match api_auth::authenticate(&state, &req).await {
        Ok(op) => op,
        Err(e) => return e.error_response(),
    };

    let fiat = &operator.fiat_currency;

    match crate::services::price_feed::usdc_to_fiat_rate(&state, fiat).await {
        Ok(rate) => HttpResponse::Ok().json(RatesResponse {
            usdc_to_fiat: rate,
            fiat_currency: fiat.clone(),
        }),
        Err(e) => e.error_response(),
    }
}
