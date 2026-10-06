use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub phone: String,
    pub public_key: String,
    pub encrypted_keypair: Vec<u8>,
    pub encryption_nonce: Vec<u8>,
    pub pin_hash: String,
    pub kyc_tier: i16,
    pub is_active: bool,
    pub operator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Submitted,
    Confirmed,
    Failed,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Submitted => write!(f, "submitted"),
            Self::Confirmed => write!(f, "confirmed"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Transfer,
    MerchantPayment,
    BillPayment,
    Swap,
    Deposit,
    Withdrawal,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transfer => write!(f, "transfer"),
            Self::MerchantPayment => write!(f, "merchant_payment"),
            Self::BillPayment => write!(f, "bill_payment"),
            Self::Swap => write!(f, "swap"),
            Self::Deposit => write!(f, "deposit"),
            Self::Withdrawal => write!(f, "withdrawal"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub idempotency_key: String,
    pub tx_type: String,
    pub status: String,
    pub amount: f64,
    pub asset_code: String,
    pub stellar_tx_hash: Option<String>,
    pub recipient_phone: Option<String>,
    pub recipient_public_key: Option<String>,
    pub merchant_code: Option<String>,
    pub memo: Option<String>,
    pub error_detail: Option<String>,
    pub operator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    pub id: Uuid,
    pub merchant_code: String,
    pub name: String,
    pub public_key: String,
    pub is_active: bool,
    pub operator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ref_id: String,
    pub action: String,
    pub detail: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
