/// Validate a 4-digit PIN format
pub fn is_valid_pin(pin: &str) -> bool {
    pin.len() == 4 && pin.chars().all(|c| c.is_ascii_digit())
}

/// Validate a positive monetary amount
pub fn is_valid_amount(input: &str) -> Option<f64> {
    let amount: f64 = input.parse().ok()?;
    if amount > 0.0 && amount < 1_000_000.0 {
        Some(amount)
    } else {
        None
    }
}

/// Validate a merchant code format (alphanumeric, 4-10 chars)
pub fn is_valid_merchant_code(code: &str) -> bool {
    let len = code.len();
    (4..=10).contains(&len) && code.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Validate a Stellar public key format (starts with G, 56 chars)
pub fn is_valid_stellar_key(key: &str) -> bool {
    key.len() == 56 && key.starts_with('G')
}
