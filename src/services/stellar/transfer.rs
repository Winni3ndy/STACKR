use stellar_xdr::curr::Memo;

use crate::context::RequestContext;
use crate::error::AppError;
use crate::services::idempotency;

use super::tx_builder;

/// Send USDC from one Stackr user (identified by phone) to a destination Stellar address.
/// Returns the Stellar transaction hash on success.
pub async fn send_usdc(
    ctx: &RequestContext,
    sender_phone: &str,
    destination_key: &str,
    amount: f64,
) -> Result<String, AppError> {
    // Generate idempotency key (prefixed with operator_id)
    let idemp_key = format!(
        "{}:transfer:{}:{}:{}:{}",
        ctx.operator.id,
        sender_phone,
        destination_key,
        amount,
        chrono::Utc::now().timestamp() / 10 // 10-second dedup window
    );

    // Check idempotency — if this exact transfer was already processed, return the hash
    if let Some(existing_hash) = idempotency::check(&ctx.state, &idemp_key).await? {
        return Ok(existing_hash);
    }

    // Load sender's signing key from encrypted storage
    let signing_key = super::wallet::load_signing_key(ctx, sender_phone).await?;

    // Build the USDC asset
    let usdc = tx_builder::build_asset(&ctx.operator.usdc_asset_code, &ctx.operator.usdc_issuer)?;

    // Build payment operation
    let payment_op = tx_builder::op_payment(destination_key, usdc, tx_builder::to_stroops(amount))?;

    // Build, sign, and submit in one call
    let tx_hash = tx_builder::build_sign_submit(
        ctx,
        &signing_key,
        vec![payment_op],
        tx_builder::memo_text("stackr")?,
    )
    .await?;

    // Record idempotency
    idempotency::record(&ctx.state, &idemp_key, &tx_hash).await?;

    // Record transaction in DB
    let db = ctx.state.db_pool.get().await?;
    db.execute(
        "INSERT INTO transactions (user_id, idempotency_key, tx_type, status, amount, asset_code, stellar_tx_hash, recipient_public_key, operator_id)
         SELECT id, $2, 'transfer', 'confirmed', $3, $4, $5, $6, $7
         FROM users WHERE phone = $1 AND operator_id = $7",
        &[
            &sender_phone,
            &idemp_key,
            &amount,
            &ctx.operator.usdc_asset_code,
            &tx_hash,
            &destination_key,
            &ctx.operator.id,
        ],
    )
    .await?;

    Ok(tx_hash)
}

/// Send native XLM between accounts.
pub async fn send_xlm(
    ctx: &RequestContext,
    sender_phone: &str,
    destination_key: &str,
    amount: f64,
) -> Result<String, AppError> {
    let signing_key = super::wallet::load_signing_key(ctx, sender_phone).await?;

    let payment_op = tx_builder::op_payment(
        destination_key,
        stellar_xdr::curr::Asset::Native,
        tx_builder::to_stroops(amount),
    )?;

    let tx_hash =
        tx_builder::build_sign_submit(ctx, &signing_key, vec![payment_op], Memo::None).await?;

    Ok(tx_hash)
}
