use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    // Client errors
    InvalidInput(String),
    Unauthorized(String),
    NotFound(String),
    RateLimited,
    InsufficientBalance { available: f64, requested: f64 },
    DailyLimitExceeded { limit: f64, used: f64 },
    InvalidPin,
    AccountNotFound(String),

    // Stellar errors
    StellarSubmitFailed(String),
    StellarNetworkError(String),
    TrustlineRequired(String),

    // Infrastructure errors
    DatabaseError(String),
    RedisError(String),
    ExternalApiError { service: String, detail: String },

    // Internal
    Internal(String),
    WalletDerivationError(String),
    EncryptionError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InsufficientBalance {
                available,
                requested,
            } => {
                write!(
                    f,
                    "Insufficient balance: have {available}, need {requested}"
                )
            }
            Self::DailyLimitExceeded { limit, used } => {
                write!(f, "Daily limit exceeded: limit {limit}, used {used}")
            }
            Self::InvalidPin => write!(f, "Invalid PIN"),
            Self::AccountNotFound(phone) => write!(f, "Account not found: {phone}"),
            Self::StellarSubmitFailed(msg) => write!(f, "Stellar submit failed: {msg}"),
            Self::StellarNetworkError(msg) => write!(f, "Stellar network error: {msg}"),
            Self::TrustlineRequired(asset) => write!(f, "Trustline required for {asset}"),
            Self::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            Self::RedisError(msg) => write!(f, "Redis error: {msg}"),
            Self::ExternalApiError { service, detail } => {
                write!(f, "{service} API error: {detail}")
            }
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::WalletDerivationError(msg) => write!(f, "Wallet derivation error: {msg}"),
            Self::EncryptionError(msg) => write!(f, "Encryption error: {msg}"),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        // For USSD, errors get mapped to user-friendly text in the USSD menu layer.
        // This HTTP response is for non-USSD routes (webhooks, health, etc.)
        match self {
            Self::InvalidInput(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({"error": msg}))
            }
            Self::Unauthorized(_) => {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "unauthorized"}))
            }
            Self::NotFound(_) => {
                HttpResponse::NotFound().json(serde_json::json!({"error": "not found"}))
            }
            Self::RateLimited => {
                HttpResponse::TooManyRequests().json(serde_json::json!({"error": "rate limited"}))
            }
            _ => {
                // Never leak internal details to clients
                tracing::error!(error = %self, "Internal server error");
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "internal server error"}))
            }
        }
    }
}

impl AppError {
    /// Convert error to a USSD-friendly message for display on feature phones
    pub fn to_ussd_message(&self) -> &str {
        match self {
            Self::InvalidInput(_) => "Invalid input. Please try again.",
            Self::InsufficientBalance { .. } => "Insufficient balance for this transaction.",
            Self::DailyLimitExceeded { .. } => "Daily transaction limit exceeded.",
            Self::InvalidPin => "Incorrect PIN. Please try again.",
            Self::AccountNotFound(_) => "Account not found. Dial to register first.",
            Self::RateLimited => "Too many requests. Please try again shortly.",
            Self::TrustlineRequired(_) => "Setting up your account. Please try again.",
            _ => "Service temporarily unavailable. Please try again later.",
        }
    }
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        Self::DatabaseError(e.to_string())
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(e: tokio_postgres::Error) -> Self {
        Self::DatabaseError(e.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        Self::RedisError(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::ExternalApiError {
            service: "HTTP".to_string(),
            detail: e.to_string(),
        }
    }
}
