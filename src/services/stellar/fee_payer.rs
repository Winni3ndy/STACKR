use ed25519_dalek::SigningKey;
use tracing::info;

use crate::context::RequestContext;
use crate::error::AppError;

use super::tx_builder;

/// Decode the fee payer's secret key from operator config (Stellar secret key format: S...).
fn load_fee_payer_key(ctx: &RequestContext) -> Result<SigningKey, AppError> {
    let secret = stellar_strkey::ed25519::PrivateKey::from_string(&ctx.operator.fee_payer_secret)
        .map_err(|e| AppError::Internal(format!("Invalid FEE_PAYER_SECRET: {e}")))?;
    Ok(SigningKey::from_bytes(&secret.0))
}

/// Fund a new Stellar account with the minimum balance from the fee payer account.
/// Creates the account on the Stellar network via a create_account operation.
///
/// Starting balance: 2.5 XLM — enough for base reserve + 2 trustlines + tx fees.
pub async fn fund_new_account(
    ctx: &RequestContext,
    destination_key: &str,
) -> Result<String, AppError> {
    let fee_payer = load_fee_payer_key(ctx)?;

    // 2.5 XLM covers: 1 XLM base reserve + 0.5 XLM × 2 trustlines + buffer for fees
    let starting_balance_stroops = tx_builder::to_stroops(2.5)?;

    let create_op = tx_builder::op_create_account(destination_key, starting_balance_stroops)?;

    let tx_hash = tx_builder::build_sign_submit(
        ctx,
        &fee_payer,
        vec![create_op],
        tx_builder::memo_text("stackr-new")?,
    )
    .await?;

    info!(
        destination = %destination_key,
        tx_hash = %tx_hash,
        "Funded new account with 2.5 XLM"
    );

    Ok(tx_hash)
}

/// Send USDC from the fee payer (platform treasury) to a destination.
/// Used for refunds when a bill payment API call fails after debiting the user.
pub async fn send_from_fee_payer(
    ctx: &RequestContext,
    destination_key: &str,
    usdc_amount: f64,
) -> Result<String, AppError> {
    let fee_payer = load_fee_payer_key(ctx)?;

    let usdc = tx_builder::build_asset(&ctx.operator.usdc_asset_code, &ctx.operator.usdc_issuer)?;

    let payment_op =
        tx_builder::op_payment(destination_key, usdc, tx_builder::to_stroops(usdc_amount)?)?;

    let tx_hash = tx_builder::build_sign_submit(
        ctx,
        &fee_payer,
        vec![payment_op],
        tx_builder::memo_text("stackr-refund")?,
    )
    .await?;

    info!(
        destination = %destination_key,
        amount = usdc_amount,
        tx_hash = %tx_hash,
        "Fee payer sent USDC refund"
    );

    Ok(tx_hash)
}

/// Sponsor a transaction for a user (fee bumping).
/// Wraps the user's signed transaction in a FeeBumpTransactionEnvelope
/// so the fee payer covers the network fee instead of the user.
pub async fn sponsor_fee(ctx: &RequestContext, inner_tx_xdr: &str) -> Result<String, AppError> {
    use ed25519_dalek::Signer;
    use sha2::{Digest, Sha256};
    use stellar_xdr::curr::{
        DecoratedSignature, FeeBumpTransaction, FeeBumpTransactionEnvelope, FeeBumpTransactionExt,
        FeeBumpTransactionInnerTx, Limits, ReadXdr, Signature, SignatureHint, TransactionEnvelope,
        WriteXdr,
    };

    let fee_payer = load_fee_payer_key(ctx)?;
    let fee_payer_muxed = tx_builder::signing_key_to_muxed(&fee_payer);

    // Decode the inner transaction
    let inner_envelope = TransactionEnvelope::from_xdr_base64(inner_tx_xdr, Limits::none())
        .map_err(|e| AppError::Internal(format!("Invalid inner TX XDR: {e}")))?;

    let inner_tx = match inner_envelope {
        TransactionEnvelope::Tx(ref env) => FeeBumpTransactionInnerTx::Tx(env.clone()),
        _ => {
            return Err(AppError::Internal(
                "Fee bump requires a V1 transaction envelope".to_string(),
            ))
        }
    };

    // Get the inner fee and bump it (at least 2x the inner fee)
    let inner_fee = match &inner_envelope {
        TransactionEnvelope::Tx(env) => env.tx.fee,
        _ => 100,
    };
    let bumped_fee = std::cmp::max(inner_fee * 2, 200);

    let fee_bump_tx = FeeBumpTransaction {
        fee_source: fee_payer_muxed,
        fee: bumped_fee as i64,
        inner_tx,
        ext: FeeBumpTransactionExt::V0,
    };

    // Sign the fee bump transaction
    let network_id = Sha256::digest(ctx.operator.stellar_network_passphrase.as_bytes());
    let tx_xdr = fee_bump_tx
        .to_xdr(Limits::none())
        .map_err(|e| AppError::Internal(format!("XDR error: {e}")))?;

    let mut payload = Vec::with_capacity(36 + tx_xdr.len());
    payload.extend_from_slice(&network_id);
    // ENVELOPE_TYPE_TX_FEE_BUMP = 5
    payload.extend_from_slice(&5u32.to_be_bytes());
    payload.extend_from_slice(&tx_xdr);

    let tx_hash_bytes = Sha256::digest(&payload);
    let signature = fee_payer.sign(&tx_hash_bytes);

    let pk_bytes = fee_payer.verifying_key().to_bytes();
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

    let envelope = TransactionEnvelope::TxFeeBump(FeeBumpTransactionEnvelope {
        tx: fee_bump_tx,
        signatures: vec![decorated_sig]
            .try_into()
            .map_err(|_| AppError::Internal("Signature list failed".to_string()))?,
    });

    let envelope_b64 = envelope
        .to_xdr_base64(Limits::none())
        .map_err(|e| AppError::Internal(format!("Base64 encoding failed: {e}")))?;

    let response = super::client::submit_transaction(ctx, &envelope_b64).await?;

    info!(tx_hash = %response.hash, "Fee bump transaction submitted");

    Ok(response.hash)
}
