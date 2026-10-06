use std::net::IpAddr;

use actix_web::HttpRequest;

use crate::config::AppConfig;

/// Extract the real client IP, respecting trusted proxies.
/// If the peer IP is in TRUSTED_PROXY_IPS, use X-Forwarded-For.
/// Otherwise, use the direct peer IP.
pub fn extract_client_ip(req: &HttpRequest, config: &AppConfig) -> Option<IpAddr> {
    let peer_ip = req.peer_addr()?.ip();

    if config.trusted_proxy_ips.contains(&peer_ip) {
        // Trust X-Forwarded-For from known proxies
        if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
            if let Ok(val) = forwarded.to_str() {
                // Take the first (leftmost) IP — the original client
                if let Some(first) = val.split(',').next() {
                    if let Ok(ip) = first.trim().parse::<IpAddr>() {
                        return Some(ip);
                    }
                }
            }
        }
    }

    Some(peer_ip)
}

/// Check if the request IP is in the Africa's Talking allowlist.
/// In development mode with an empty allowlist, all IPs are allowed.
pub fn is_at_ip_allowed(client_ip: IpAddr, config: &AppConfig) -> bool {
    if config.at_allowed_ips.is_empty() && !config.is_production() {
        return true;
    }
    config.at_allowed_ips.contains(&client_ip)
}
