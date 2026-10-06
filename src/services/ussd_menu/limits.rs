//! Shared KYC daily limit enforcement for all transactional USSD flows.

use crate::context::RequestContext;

/// Check if a user can spend `amount` USDC without exceeding their KYC daily limit.
/// Returns Ok(()) if allowed, or Err(user-friendly message) if over limit.
pub async fn check_daily_limit(
    ctx: &RequestContext,
    phone: &str,
    amount: f64,
) -> Result<(), String> {
    let db = ctx
        .state
        .db_pool
        .get()
        .await
        .map_err(|_| "Service temporarily unavailable.".to_string())?;

    let row = db
        .query_one(
            "SELECT kyc_tier, daily_spent, daily_spent_date FROM users WHERE phone = $1 AND operator_id = $2",
            &[&phone, &ctx.operator.id],
        )
        .await
        .map_err(|_| "Account error.".to_string())?;

    let kyc_tier: i16 = row.get("kyc_tier");
    let mut daily_spent: f64 = row.get("daily_spent");
    let spent_date: chrono::NaiveDate = row.get("daily_spent_date");

    // Reset if new day
    if spent_date != chrono::Utc::now().date_naive() {
        daily_spent = 0.0;
    }

    let limit = match kyc_tier {
        0 => ctx.operator.kyc_tier0_daily_limit,
        1 => ctx.operator.kyc_tier1_daily_limit,
        _ => ctx.operator.kyc_tier2_daily_limit,
    };

    if daily_spent + amount > limit {
        return Err(format!(
            "Daily limit exceeded.\nLimit: {:.2} USDC\nUsed: {:.2} USDC",
            limit, daily_spent
        ));
    }

    Ok(())
}

/// Update the user's daily spend total after a successful transaction.
pub async fn update_daily_spend(ctx: &RequestContext, phone: &str, amount: f64) -> Result<(), ()> {
    let db = ctx.state.db_pool.get().await.map_err(|_| ())?;
    let today = chrono::Utc::now().date_naive();

    db.execute(
        "UPDATE users SET daily_spent = CASE
            WHEN daily_spent_date = $1 THEN daily_spent + $2
            ELSE $2
         END,
         daily_spent_date = $1,
         updated_at = NOW()
         WHERE phone = $3 AND operator_id = $4",
        &[&today, &amount, &phone, &ctx.operator.id],
    )
    .await
    .map_err(|_| ())?;

    Ok(())
}
