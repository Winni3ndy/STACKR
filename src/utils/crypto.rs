use subtle::ConstantTimeEq;

/// Constant-time comparison for secrets (API keys, webhook secrets, etc.)
/// Prevents timing attacks on secret validation.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).unwrap_u8() == 1
}

/// Generate a cryptographically random hex string of the given byte length.
pub fn random_hex(byte_len: usize) -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; byte_len];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}
