use uuid::Uuid;

/// Generate a unique request ID for tracing.
/// If the request includes an X-Request-Id header, use that; otherwise generate one.
pub fn get_or_generate(req: &actix_web::HttpRequest) -> String {
    req.headers()
        .get("X-Request-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}
