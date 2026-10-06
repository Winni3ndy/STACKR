use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use ed25519_dalek::SigningKey;
use hkdf::Hkdf;
use rand::rngs::OsRng;
use sha2::Sha256;
use tracing::info;

use crate::context::RequestContext;
use crate::error::AppError;

/// Derive a deterministic Ed25519 keypair from a master seed + phone number
/// using HKDF-SHA256. The same phone always produces the same keypair.
pub fn derive_keypair(master_seed: &str, phone: &str) -> Result<SigningKey, AppError> {
    let master_bytes = master_seed.as_bytes();

    let hkdf = Hkdf::<Sha256>::new(Some(phone.as_bytes()), master_bytes);

    let mut okm = [0u8; 32];
    hkdf.expand(b"stackr-stellar-wallet-v1", &mut okm)
        .map_err(|e| AppError::WalletDerivationError(e.to_string()))?;

    Ok(SigningKey::from_bytes(&okm))
}

/// Encrypt a keypair's secret bytes using AES-256-GCM with a random 12-byte nonce.
/// Returns (ciphertext, nonce).
pub fn encrypt_keypair(
    encryption_key: &str,
    secret_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let key_bytes = hex::decode(encryption_key)
        .map_err(|e| AppError::EncryptionError(format!("Invalid encryption key hex: {e}")))?;

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::EncryptionError(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::RngCore::fill_bytes(&mut OsRng, &mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, secret_bytes)
        .map_err(|e| AppError::EncryptionError(e.to_string()))?;

    Ok((ciphertext, nonce_bytes.to_vec()))
}

/// Decrypt a keypair's secret bytes from AES-256-GCM ciphertext + nonce.
pub fn decrypt_keypair(
    encryption_key: &str,
    ciphertext: &[u8],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>, AppError> {
    let key_bytes = hex::decode(encryption_key)
        .map_err(|e| AppError::EncryptionError(format!("Invalid encryption key hex: {e}")))?;

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| AppError::EncryptionError(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::EncryptionError(e.to_string()))
}

/// Get the Stellar public key (G...) from an Ed25519 signing key
pub fn public_key_to_stellar(signing_key: &SigningKey) -> String {
    let verifying_key = signing_key.verifying_key();
    stellar_strkey::ed25519::PublicKey(verifying_key.to_bytes()).to_string()
}

/// Get the fee payer's public key (G...) from operator config.
pub fn fee_payer_public_key(ctx: &RequestContext) -> Result<String, AppError> {
    let secret = stellar_strkey::ed25519::PrivateKey::from_string(&ctx.operator.fee_payer_secret)
        .map_err(|e| AppError::Internal(format!("Invalid FEE_PAYER_SECRET: {e}")))?;
    let signing_key = SigningKey::from_bytes(&secret.0);
    Ok(public_key_to_stellar(&signing_key))
}

/// Create a new user wallet: derive keypair, encrypt, store in DB, fund via fee payer.
pub async fn create_user_wallet(
    ctx: &RequestContext,
    phone: &str,
    pin: &str,
) -> Result<String, AppError> {
    // Derive deterministic keypair
    let signing_key = derive_keypair(&ctx.operator.wallet_master_seed, phone)?;
    let public_key = public_key_to_stellar(&signing_key);

    // Encrypt the secret key for storage
    let (encrypted, nonce) =
        encrypt_keypair(&ctx.operator.wallet_encryption_key, signing_key.as_bytes())?;

    // Hash the PIN with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pin_hash = argon2
        .hash_password(pin.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("PIN hashing failed: {e}")))?
        .to_string();

    // Store in database
    let db = ctx.state.db_pool.get().await?;
    db.execute(
        "INSERT INTO users (phone, public_key, encrypted_keypair, encryption_nonce, pin_hash, operator_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (operator_id, phone) DO NOTHING",
        &[
            &phone,
            &public_key,
            &encrypted,
            &nonce,
            &pin_hash,
            &ctx.operator.id,
        ],
    )
    .await?;

    // Fund the account via fee payer (create account operation)
    if let Err(e) = super::fee_payer::fund_new_account(ctx, &public_key).await {
        tracing::warn!(error = %e, public_key = %public_key, "Failed to fund new account — user can retry");
    }

    // Set up USDC trustline
    if let Err(e) = super::trustlines::ensure_usdc_trustline(ctx, phone).await {
        tracing::warn!(error = %e, "Failed to set USDC trustline — will retry on first transaction");
    }

    info!(phone = %phone, public_key = %public_key, "User wallet created");

    Ok(public_key)
}

/// Load a user's signing key from the database (decrypt stored keypair)
pub async fn load_signing_key(ctx: &RequestContext, phone: &str) -> Result<SigningKey, AppError> {
    let db = ctx.state.db_pool.get().await?;

    let row = db
        .query_opt(
            "SELECT encrypted_keypair, encryption_nonce FROM users
             WHERE phone = $1 AND operator_id = $2 AND is_active = TRUE",
            &[&phone, &ctx.operator.id],
        )
        .await?
        .ok_or_else(|| AppError::AccountNotFound(phone.to_string()))?;

    let encrypted: Vec<u8> = row.get("encrypted_keypair");
    let nonce: Vec<u8> = row.get("encryption_nonce");

    let secret_bytes = decrypt_keypair(&ctx.operator.wallet_encryption_key, &encrypted, &nonce)?;

    let key_bytes: [u8; 32] = secret_bytes
        .try_into()
        .map_err(|_| AppError::WalletDerivationError("Invalid key length".to_string()))?;

    Ok(SigningKey::from_bytes(&key_bytes))
}
