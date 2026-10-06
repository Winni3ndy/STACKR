//! Stellar transaction builder: construct, sign, and submit XDR transaction envelopes.
//!
//! All Stellar operations (payment, create_account, change_trust, path_payment)
//! go through this module to build proper XDR, sign with ed25519, and submit to Horizon.

use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use stellar_xdr::curr::{
    AccountId, AlphaNum4, Asset, AssetCode4, ChangeTrustAsset, ChangeTrustOp, CreateAccountOp,
    DecoratedSignature, Limits, Memo, MuxedAccount, Operation, OperationBody,
    PathPaymentStrictSendOp, PaymentOp, Preconditions, PublicKey, SequenceNumber, Signature,
    SignatureHint, StringM, TimeBounds, TimePoint, Transaction, TransactionEnvelope,
    TransactionV1Envelope, Uint256, VecM, WriteXdr,
};

use crate::context::RequestContext;
use crate::error::AppError;

/// Convert a Stellar public key string (G...) to a Uint256 (raw 32-byte Ed25519 key)
pub fn stellar_address_to_uint256(address: &str) -> Result<Uint256, AppError> {
    let strkey = stellar_strkey::ed25519::PublicKey::from_string(address)
        .map_err(|e| AppError::InvalidInput(format!("Invalid Stellar address: {e}")))?;
    Ok(Uint256(strkey.0))
}

/// Convert a Stellar public key string to AccountId
pub fn stellar_address_to_account_id(address: &str) -> Result<AccountId, AppError> {
    let uint256 = stellar_address_to_uint256(address)?;
    Ok(AccountId(PublicKey::PublicKeyTypeEd25519(uint256)))
}

/// Convert a Stellar public key string to MuxedAccount
pub fn stellar_address_to_muxed(address: &str) -> Result<MuxedAccount, AppError> {
    let uint256 = stellar_address_to_uint256(address)?;
    Ok(MuxedAccount::Ed25519(uint256))
}

/// Convert an ed25519_dalek SigningKey to a Stellar MuxedAccount
pub fn signing_key_to_muxed(key: &SigningKey) -> MuxedAccount {
    let verifying = key.verifying_key();
    MuxedAccount::Ed25519(Uint256(verifying.to_bytes()))
}

/// Convert an ed25519_dalek SigningKey to a Stellar AccountId
pub fn signing_key_to_account_id(key: &SigningKey) -> AccountId {
    let verifying = key.verifying_key();
    AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
        verifying.to_bytes(),
    )))
}

/// Build a Stellar Asset from code + issuer
pub fn build_asset(code: &str, issuer: &str) -> Result<Asset, AppError> {
    if code == "XLM" || code == "native" {
        return Ok(Asset::Native);
    }

    if code.len() <= 4 {
        let mut code_bytes = [0u8; 4];
        code_bytes[..code.len()].copy_from_slice(code.as_bytes());

        let issuer_account = stellar_address_to_account_id(issuer)?;

        Ok(Asset::CreditAlphanum4(AlphaNum4 {
            asset_code: AssetCode4(code_bytes),
            issuer: issuer_account,
        }))
    } else {
        // AssetCode12 for longer codes
        Err(AppError::InvalidInput(format!(
            "Asset codes > 4 chars not yet supported: {code}"
        )))
    }
}

/// Convert a human-readable amount (e.g., 10.50) to stroops (Stellar's smallest unit).
/// 1 XLM/USDC = 10,000,000 stroops
pub fn to_stroops(amount: f64) -> Result<i64, AppError> {
    if !amount.is_finite() || amount < 0.0 {
        return Err(AppError::InvalidInput(format!(
            "Invalid amount for stroops conversion: {amount}"
        )));
    }
    let stroops = amount * 10_000_000.0;
    if stroops > i64::MAX as f64 {
        return Err(AppError::InvalidInput(format!(
            "Amount too large for stroops: {amount}"
        )));
    }
    Ok(stroops as i64)
}

/// Build a complete Transaction with the given operations.
pub fn build_transaction(
    source: &MuxedAccount,
    sequence: i64,
    fee_per_op: u32,
    operations: Vec<Operation>,
    memo: Memo,
    timeout_secs: u64,
) -> Result<Transaction, AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AppError::Internal(format!("System time error: {e}")))?
        .as_secs();

    let time_bounds = TimeBounds {
        min_time: TimePoint(0),
        max_time: TimePoint(now + timeout_secs),
    };

    let ops: VecM<Operation, 100> = operations
        .try_into()
        .map_err(|_| AppError::Internal("Too many operations (max 100)".to_string()))?;

    let fee = fee_per_op * ops.len() as u32;

    Ok(Transaction {
        source_account: source.clone(),
        fee,
        seq_num: SequenceNumber(sequence),
        cond: Preconditions::Time(time_bounds),
        memo,
        operations: ops,
        ext: stellar_xdr::curr::TransactionExt::V0,
    })
}

/// Sign a transaction with the network passphrase and a signing key.
/// Returns the signed TransactionEnvelope as base64 XDR.
pub fn sign_and_encode(
    tx: Transaction,
    network_passphrase: &str,
    signer: &SigningKey,
) -> Result<String, AppError> {
    // Transaction hash = SHA256(network_id + ENVELOPE_TYPE_TX + tx_xdr)
    let network_id = Sha256::digest(network_passphrase.as_bytes());

    // The tagged payload to sign: network_id (32 bytes) + envelope_type_tx (4 bytes) + tx XDR
    let tx_xdr = tx
        .to_xdr(Limits::none())
        .map_err(|e| AppError::Internal(format!("XDR serialization failed: {e}")))?;

    let mut payload = Vec::with_capacity(36 + tx_xdr.len());
    payload.extend_from_slice(&network_id);
    // ENVELOPE_TYPE_TX = 2, encoded as 4-byte big-endian
    payload.extend_from_slice(&2u32.to_be_bytes());
    payload.extend_from_slice(&tx_xdr);

    let tx_hash = Sha256::digest(&payload);

    // Sign the hash
    let signature = signer.sign(&tx_hash);

    // Build the decorated signature (hint = last 4 bytes of public key)
    let verifying_key = signer.verifying_key();
    let pk_bytes = verifying_key.to_bytes();
    let hint = SignatureHint([pk_bytes[28], pk_bytes[29], pk_bytes[30], pk_bytes[31]]);

    let decorated_sig = DecoratedSignature {
        hint,
        signature: Signature(
            signature
                .to_bytes()
                .to_vec()
                .try_into()
                .map_err(|_| AppError::Internal("Signature encoding failed".to_string()))?,
        ),
    };

    // Wrap in TransactionV1Envelope
    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx,
        signatures: vec![decorated_sig]
            .try_into()
            .map_err(|_| AppError::Internal("Signature list failed".to_string()))?,
    });

    // Encode to base64 XDR
    envelope
        .to_xdr_base64(Limits::none())
        .map_err(|e| AppError::Internal(format!("Base64 encoding failed: {e}")))
}

/// Build, sign, and submit a transaction in one call.
/// Returns the transaction hash from Horizon.
pub async fn build_sign_submit(
    ctx: &RequestContext,
    source_key: &SigningKey,
    operations: Vec<Operation>,
    memo: Memo,
) -> Result<String, AppError> {
    let source = signing_key_to_muxed(source_key);
    let source_address = super::wallet::public_key_to_stellar(source_key);

    // Get sequence number
    let sequence = super::client::get_sequence(ctx, &source_address).await?;

    // Get base fee
    let base_fee = super::client::get_base_fee(ctx).await.unwrap_or(100);

    // Build transaction (5-minute timeout)
    let tx = build_transaction(&source, sequence + 1, base_fee, operations, memo, 300)?;

    // Sign and encode
    let tx_xdr = sign_and_encode(tx, &ctx.operator.stellar_network_passphrase, source_key)?;

    // Submit to Horizon
    let response = super::client::submit_transaction(ctx, &tx_xdr).await?;

    Ok(response.hash)
}

// ─── Operation constructors ──────────────────────────────────────────────────

/// Build a Payment operation
pub fn op_payment(
    destination: &str,
    asset: Asset,
    amount_stroops: i64,
) -> Result<Operation, AppError> {
    Ok(Operation {
        source_account: None,
        body: OperationBody::Payment(PaymentOp {
            destination: stellar_address_to_muxed(destination)?,
            asset,
            amount: amount_stroops,
        }),
    })
}

/// Build a CreateAccount operation
pub fn op_create_account(
    destination: &str,
    starting_balance_stroops: i64,
) -> Result<Operation, AppError> {
    Ok(Operation {
        source_account: None,
        body: OperationBody::CreateAccount(CreateAccountOp {
            destination: stellar_address_to_account_id(destination)?,
            starting_balance: starting_balance_stroops,
        }),
    })
}

/// Build a ChangeTrust operation
pub fn op_change_trust(asset: Asset, limit_stroops: i64) -> Result<Operation, AppError> {
    let line = match asset {
        Asset::Native => {
            return Err(AppError::InvalidInput(
                "Cannot create trustline for native XLM".to_string(),
            ))
        }
        Asset::CreditAlphanum4(ref a) => ChangeTrustAsset::CreditAlphanum4(a.clone()),
        Asset::CreditAlphanum12(ref a) => ChangeTrustAsset::CreditAlphanum12(a.clone()),
    };

    Ok(Operation {
        source_account: None,
        body: OperationBody::ChangeTrust(ChangeTrustOp {
            line,
            limit: limit_stroops,
        }),
    })
}

/// Build a PathPaymentStrictSend operation (DEX swap)
pub fn op_path_payment_strict_send(
    send_asset: Asset,
    send_amount_stroops: i64,
    destination: &str,
    dest_asset: Asset,
    dest_min_stroops: i64,
    path: Vec<Asset>,
) -> Result<Operation, AppError> {
    let path_vec: VecM<Asset, 5> = path
        .try_into()
        .map_err(|_| AppError::InvalidInput("Path too long (max 5 assets)".to_string()))?;

    Ok(Operation {
        source_account: None,
        body: OperationBody::PathPaymentStrictSend(PathPaymentStrictSendOp {
            send_asset,
            send_amount: send_amount_stroops,
            destination: stellar_address_to_muxed(destination)?,
            dest_asset,
            dest_min: dest_min_stroops,
            path: path_vec,
        }),
    })
}

/// Build a text memo
pub fn memo_text(text: &str) -> Result<Memo, AppError> {
    let s: StringM<28> = text
        .try_into()
        .map_err(|_| AppError::InvalidInput("Memo too long (max 28 bytes)".to_string()))?;
    Ok(Memo::Text(s))
}
