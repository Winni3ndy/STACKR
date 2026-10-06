//! Stellar SEP integration: SEP-10 authentication, SEP-24 interactive deposit/withdrawal.
//!
//! Adapted from Payce's PAJ ramp pattern but using the Stellar anchor protocol instead
//! of a proprietary ramp API. The webhook/order state machine pattern is the same.

use ed25519_dalek::Signer;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    DecoratedSignature, Limits, ReadXdr, Signature, SignatureHint, TransactionEnvelope, WriteXdr,
};
use tracing::{info, warn};

use crate::context::RequestContext;
use crate::error::AppError;

// ─── Stellar TOML discovery ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnchorInfo {
    pub web_auth_endpoint: String,
    pub transfer_server_sep24: String,
    pub signing_key: String,
}

/// Fetch and parse a Stellar anchor's .well-known/stellar.toml
pub async fn discover_anchor(
    ctx: &RequestContext,
    anchor_domain: &str,
) -> Result<AnchorInfo, AppError> {
    let toml_url = format!("https://{}/.well-known/stellar.toml", anchor_domain);

    let response = ctx
        .state
        .http_client
        .get(&toml_url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: format!("stellar.toml returned {}", response.status()),
        });
    }

    let toml_text = response
        .text()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?;

    let web_auth = parse_toml_value(&toml_text, "WEB_AUTH_ENDPOINT").ok_or_else(|| {
        AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: "WEB_AUTH_ENDPOINT not found in stellar.toml".to_string(),
        }
    })?;

    let transfer_server = parse_toml_value(&toml_text, "TRANSFER_SERVER_SEP0024")
        .or_else(|| parse_toml_value(&toml_text, "TRANSFER_SERVER"))
        .ok_or_else(|| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: "TRANSFER_SERVER not found in stellar.toml".to_string(),
        })?;

    let signing_key = parse_toml_value(&toml_text, "SIGNING_KEY").unwrap_or_default();

    Ok(AnchorInfo {
        web_auth_endpoint: web_auth,
        transfer_server_sep24: transfer_server,
        signing_key,
    })
}

// ─── SEP-10: Stellar Web Authentication ──────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ChallengeResponse {
    transaction: String,
    network_passphrase: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
}

/// Authenticate with a Stellar anchor via SEP-10.
/// Returns a JWT token for use with SEP-24 endpoints.
pub async fn sep10_authenticate(
    ctx: &RequestContext,
    anchor_domain: &str,
    phone: &str,
) -> Result<String, AppError> {
    let anchor_info = discover_anchor(ctx, anchor_domain).await?;
    let signing_key = super::wallet::load_signing_key(ctx, phone).await?;
    let public_key = super::wallet::public_key_to_stellar(&signing_key);

    // Step 1: Request challenge transaction
    let challenge_url = format!("{}?account={}", anchor_info.web_auth_endpoint, public_key);

    let challenge: ChallengeResponse = ctx
        .state
        .http_client
        .get(&challenge_url)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?
        .json()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: format!("Invalid challenge response: {e}"),
        })?;

    // Step 2: Decode, verify, and sign the challenge transaction
    let envelope = TransactionEnvelope::from_xdr_base64(&challenge.transaction, Limits::none())
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: format!("Invalid challenge XDR: {e}"),
        })?;

    // Extract the inner transaction for signing
    let signed_xdr =
        sign_challenge_envelope(envelope, &challenge.network_passphrase, &signing_key)?;

    // Step 3: POST signed challenge back to get JWT
    let token_response: TokenResponse = ctx
        .state
        .http_client
        .post(&anchor_info.web_auth_endpoint)
        .json(&serde_json::json!({ "transaction": signed_xdr }))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?
        .json()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: format!("Invalid token response: {e}"),
        })?;

    info!(
        anchor = %anchor_domain,
        phone = %phone,
        "SEP-10 authentication successful"
    );

    Ok(token_response.token)
}

/// Sign a SEP-10 challenge transaction envelope.
/// The challenge comes pre-signed by the anchor; we add our signature.
fn sign_challenge_envelope(
    envelope: TransactionEnvelope,
    network_passphrase: &str,
    signer: &ed25519_dalek::SigningKey,
) -> Result<String, AppError> {
    match envelope {
        TransactionEnvelope::Tx(mut env) => {
            // Hash the transaction for signing
            let network_id = Sha256::digest(network_passphrase.as_bytes());
            let tx_xdr = env
                .tx
                .to_xdr(Limits::none())
                .map_err(|e| AppError::Internal(format!("XDR error: {e}")))?;

            let mut payload = Vec::with_capacity(36 + tx_xdr.len());
            payload.extend_from_slice(&network_id);
            payload.extend_from_slice(&2u32.to_be_bytes()); // ENVELOPE_TYPE_TX
            payload.extend_from_slice(&tx_xdr);

            let tx_hash = Sha256::digest(&payload);
            let signature = signer.sign(&tx_hash);

            // Build decorated signature
            let pk_bytes = signer.verifying_key().to_bytes();
            let hint = SignatureHint([pk_bytes[28], pk_bytes[29], pk_bytes[30], pk_bytes[31]]);

            let decorated_sig = DecoratedSignature {
                hint,
                signature: Signature(
                    signature
                        .to_bytes()
                        .to_vec()
                        .try_into()
                        .map_err(|_| AppError::Internal("Sig encoding failed".to_string()))?,
                ),
            };

            // Append our signature to existing signatures
            let mut sigs: Vec<DecoratedSignature> = env.signatures.to_vec();
            sigs.push(decorated_sig);
            env.signatures = sigs
                .try_into()
                .map_err(|_| AppError::Internal("Too many signatures".to_string()))?;

            // Re-encode
            let signed_envelope = TransactionEnvelope::Tx(env);
            signed_envelope
                .to_xdr_base64(Limits::none())
                .map_err(|e| AppError::Internal(format!("Base64 error: {e}")))
        }
        _ => Err(AppError::Internal(
            "Challenge must be a V1 transaction envelope".to_string(),
        )),
    }
}

// ─── SEP-24: Interactive Deposit/Withdrawal ──────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sep24DepositResponse {
    pub id: String,
    pub url: Option<String>,
    pub status: String,
    #[serde(default)]
    pub extra_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sep24WithdrawResponse {
    pub id: String,
    pub url: Option<String>,
    pub status: String,
    pub stellar_account_id: Option<String>,
    pub stellar_memo_type: Option<String>,
    pub stellar_memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sep24TransactionStatus {
    pub id: String,
    pub status: String,
    pub status_eta: Option<u64>,
    pub amount_in: Option<String>,
    pub amount_out: Option<String>,
    pub amount_fee: Option<String>,
    pub stellar_transaction_id: Option<String>,
    pub external_transaction_id: Option<String>,
    pub message: Option<String>,
}

/// Initiate a SEP-24 interactive deposit (fiat → crypto).
/// Returns a URL the user should visit (in an app) or instructions to display via USSD.
pub async fn sep24_deposit(
    ctx: &RequestContext,
    anchor_domain: &str,
    phone: &str,
    asset_code: &str,
    amount: Option<f64>,
) -> Result<Sep24DepositResponse, AppError> {
    let jwt = sep10_authenticate(ctx, anchor_domain, phone).await?;
    let anchor_info = discover_anchor(ctx, anchor_domain).await?;

    let mut form_data = vec![("asset_code".to_string(), asset_code.to_string())];
    if let Some(amt) = amount {
        form_data.push(("amount".to_string(), format!("{:.2}", amt)));
    }

    let url = format!(
        "{}/transactions/deposit/interactive",
        anchor_info.transfer_server_sep24
    );

    let response = ctx
        .state
        .http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", jwt))
        .form(&form_data)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(anchor = %anchor_domain, body = %body, "SEP-24 deposit initiation failed");
        return Err(AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: body,
        });
    }

    let deposit: Sep24DepositResponse =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: format!("anchor:{anchor_domain}"),
                detail: format!("Invalid deposit response: {e}"),
            })?;

    info!(
        anchor = %anchor_domain,
        tx_id = %deposit.id,
        phone = %phone,
        "SEP-24 deposit initiated"
    );

    Ok(deposit)
}

/// Initiate a SEP-24 interactive withdrawal (crypto → fiat).
/// Returns anchor's Stellar account and memo to send tokens to.
pub async fn sep24_withdraw(
    ctx: &RequestContext,
    anchor_domain: &str,
    phone: &str,
    asset_code: &str,
    amount: f64,
) -> Result<Sep24WithdrawResponse, AppError> {
    let jwt = sep10_authenticate(ctx, anchor_domain, phone).await?;
    let anchor_info = discover_anchor(ctx, anchor_domain).await?;

    let url = format!(
        "{}/transactions/withdraw/interactive",
        anchor_info.transfer_server_sep24
    );

    let form_data = vec![
        ("asset_code".to_string(), asset_code.to_string()),
        ("amount".to_string(), format!("{:.7}", amount)),
    ];

    let response = ctx
        .state
        .http_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", jwt))
        .form(&form_data)
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        warn!(anchor = %anchor_domain, body = %body, "SEP-24 withdrawal initiation failed");
        return Err(AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: body,
        });
    }

    let withdraw: Sep24WithdrawResponse =
        response
            .json()
            .await
            .map_err(|e| AppError::ExternalApiError {
                service: format!("anchor:{anchor_domain}"),
                detail: format!("Invalid withdraw response: {e}"),
            })?;

    info!(
        anchor = %anchor_domain,
        tx_id = %withdraw.id,
        phone = %phone,
        "SEP-24 withdrawal initiated"
    );

    Ok(withdraw)
}

/// Poll a SEP-24 transaction status.
pub async fn sep24_get_transaction(
    ctx: &RequestContext,
    anchor_domain: &str,
    phone: &str,
    transaction_id: &str,
) -> Result<Sep24TransactionStatus, AppError> {
    let jwt = sep10_authenticate(ctx, anchor_domain, phone).await?;
    let anchor_info = discover_anchor(ctx, anchor_domain).await?;

    let url = format!(
        "{}/transaction?id={}",
        anchor_info.transfer_server_sep24, transaction_id
    );

    let response = ctx
        .state
        .http_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", jwt))
        .send()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: e.to_string(),
        })?;

    #[derive(Deserialize)]
    struct Wrapper {
        transaction: Sep24TransactionStatus,
    }

    let wrapper: Wrapper = response
        .json()
        .await
        .map_err(|e| AppError::ExternalApiError {
            service: format!("anchor:{anchor_domain}"),
            detail: format!("Invalid transaction response: {e}"),
        })?;

    Ok(wrapper.transaction)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Simple stellar.toml value parser (avoids a full TOML crate dependency)
fn parse_toml_value(toml: &str, key: &str) -> Option<String> {
    for line in toml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let value = rest.trim().trim_matches('"').to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}
