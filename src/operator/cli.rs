use anyhow::Result;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AppState;

/// Parameters for creating a new operator.
pub struct CreateOperatorParams {
    pub slug: String,
    pub name: String,
    pub fiat_currency: String,
    pub fiat_country: String,
    pub wallet_master_seed: String,
    pub wallet_encryption_key: String,
    pub fee_payer_secret: String,
    pub usdc_asset_code: String,
    pub usdc_issuer: String,
    pub stellar_horizon_url: String,
    pub stellar_network_passphrase: String,
    pub anchor_domain: String,
}

/// Create a new operator and return its API key (printed once, never stored in plaintext).
pub async fn create_operator(
    state: &AppState,
    params: &CreateOperatorParams,
) -> Result<(Uuid, String)> {
    let db = state.db_pool.get().await?;

    let row = db
        .query_one(
            "INSERT INTO operators (
                slug, name, is_active,
                fiat_currency, fiat_country,
                wallet_master_seed, wallet_encryption_key,
                fee_payer_secret, usdc_asset_code, usdc_issuer,
                stellar_horizon_url, stellar_network_passphrase,
                anchor_domain
            ) VALUES (
                $1, $2, TRUE,
                $3, $4,
                $5, $6,
                $7, $8, $9,
                $10, $11,
                $12
            ) RETURNING id",
            &[
                &params.slug.as_str(),
                &params.name.as_str(),
                &params.fiat_currency.as_str(),
                &params.fiat_country.as_str(),
                &params.wallet_master_seed.as_str(),
                &params.wallet_encryption_key.as_str(),
                &params.fee_payer_secret.as_str(),
                &params.usdc_asset_code.as_str(),
                &params.usdc_issuer.as_str(),
                &params.stellar_horizon_url.as_str(),
                &params.stellar_network_passphrase.as_str(),
                &params.anchor_domain.as_str(),
            ],
        )
        .await?;

    let operator_id: Uuid = row.get("id");

    // Generate API key
    let api_key = generate_api_key();
    let key_hash = hex::encode(Sha256::digest(api_key.as_bytes()));
    let key_prefix = &api_key[..std::cmp::min(12, api_key.len())];

    db.execute(
        "INSERT INTO api_keys (operator_id, key_hash, key_prefix, label, is_active)
         VALUES ($1, $2, $3, $4, TRUE)",
        &[
            &operator_id,
            &key_hash,
            &key_prefix,
            &format!("{} default key", params.slug),
        ],
    )
    .await?;

    Ok((operator_id, api_key))
}

/// Generate a new API key for an existing operator.
pub async fn generate_api_key_for_operator(
    state: &AppState,
    operator_id: Uuid,
    label: &str,
) -> Result<String> {
    let db = state.db_pool.get().await?;

    // Verify operator exists
    let exists: bool = db
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM operators WHERE id = $1 AND is_active = TRUE)",
            &[&operator_id],
        )
        .await?
        .get(0);

    if !exists {
        anyhow::bail!("Operator not found or inactive: {}", operator_id);
    }

    let api_key = generate_api_key();
    let key_hash = hex::encode(Sha256::digest(api_key.as_bytes()));
    let key_prefix = &api_key[..std::cmp::min(12, api_key.len())];

    db.execute(
        "INSERT INTO api_keys (operator_id, key_hash, key_prefix, label, is_active)
         VALUES ($1, $2, $3, $4, TRUE)",
        &[&operator_id, &key_hash, &key_prefix, &label],
    )
    .await?;

    Ok(api_key)
}

/// Generate a cryptographically random API key with `sk_` prefix.
fn generate_api_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    format!("sk_{}", hex::encode(bytes))
}
