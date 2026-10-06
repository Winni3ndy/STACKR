use std::net::IpAddr;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::AppError;

/// Sliding-window rate limiter backed by Redis INCR + EXPIRE.
/// Returns Ok(remaining) if allowed, Err(AppError::RateLimited) if exceeded.
pub async fn check_rate_limit(
    redis: &mut ConnectionManager,
    ip: IpAddr,
    max_requests: u64,
    window_secs: u64,
) -> Result<u64, AppError> {
    let key = format!("rl:{}", ip);

    let count: u64 = redis.incr(&key, 1u64).await?;

    if count == 1 {
        // First request in window — set expiry
        let _: () = redis.expire(&key, window_secs as i64).await?;
    }

    if count > max_requests {
        return Err(AppError::RateLimited);
    }

    Ok(max_requests - count)
}
