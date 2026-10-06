use stellar_xdr::curr::Memo;
use tracing::info;

use crate::context::RequestContext;
use crate::error::AppError;

use super::tx_builder;

/// Ensure the user's Stellar account has a USDC trustline.
/// If not, create one automatically (signed by user's key).
pub async fn ensure_usdc_trustline(ctx: &RequestContext, phone: &str) -> Result<(), AppError> {
    ensure_trustline(
        ctx,
        phone,
        &ctx.operator.usdc_asset_code.clone(),
        &ctx.operator.usdc_issuer.clone(),
    )
    .await
}

/// Ensure trustline for any arbitrary asset.
/// Checks if the trustline exists; if not, builds a ChangeTrust operation and submits it.
pub async fn ensure_trustline(
    ctx: &RequestContext,
    phone: &str,
    asset_code: &str,
    asset_issuer: &str,
) -> Result<(), AppError> {
    let signing_key = super::wallet::load_signing_key(ctx, phone).await?;
    let public_key = super::wallet::public_key_to_stellar(&signing_key);

    // Check if trustline already exists
    let account = super::client::get_account(ctx, &public_key).await?;

    let has_asset = account.balances.iter().any(|b| {
        b.asset_code.as_deref() == Some(asset_code)
            && b.asset_issuer.as_deref() == Some(asset_issuer)
    });

    if has_asset {
        return Ok(());
    }

    info!(phone = %phone, asset = %asset_code, "Creating trustline");

    // Build the asset
    let asset = tx_builder::build_asset(asset_code, asset_issuer)?;

    // Max trustline limit (effectively unlimited)
    let max_limit = tx_builder::to_stroops(922337203685.4775)?; // i64::MAX / 10_000_000

    // Build ChangeTrust operation
    let trust_op = tx_builder::op_change_trust(asset, max_limit)?;

    // Build, sign, and submit
    let tx_hash =
        tx_builder::build_sign_submit(ctx, &signing_key, vec![trust_op], Memo::None).await?;

    info!(
        phone = %phone,
        asset = %asset_code,
        tx_hash = %tx_hash,
        "Trustline created"
    );

    Ok(())
}

/// Remove a trustline (set limit to 0). Only works if the balance is 0.
pub async fn remove_trustline(
    ctx: &RequestContext,
    phone: &str,
    asset_code: &str,
    asset_issuer: &str,
) -> Result<(), AppError> {
    let signing_key = super::wallet::load_signing_key(ctx, phone).await?;

    let asset = tx_builder::build_asset(asset_code, asset_issuer)?;

    let trust_op = tx_builder::op_change_trust(asset, 0)?;

    let tx_hash =
        tx_builder::build_sign_submit(ctx, &signing_key, vec![trust_op], Memo::None).await?;

    info!(
        phone = %phone,
        asset = %asset_code,
        tx_hash = %tx_hash,
        "Trustline removed"
    );

    Ok(())
}
