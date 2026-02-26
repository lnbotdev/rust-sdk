//! Request and response types for the LnBot API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

/// A wallet's current state.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WalletResponse {
    pub wallet_id: String,
    pub name: String,
    pub balance: i64,
    pub on_hold: i64,
    pub available: i64,
}

/// Parameters for creating a new wallet.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateWalletRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Parameters for updating a wallet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWalletRequest {
    pub name: String,
}

impl UpdateWalletRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Response from creating a new wallet, including credentials.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateWalletResponse {
    pub wallet_id: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub name: String,
    pub address: String,
    pub recovery_passphrase: String,
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

/// An API key's metadata (the actual key value is never returned after creation).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApiKeyResponse {
    pub id: String,
    pub name: String,
    pub hint: String,
    pub created_at: Option<String>,
    pub last_used_at: Option<String>,
}

/// Response from rotating an API key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RotateApiKeyResponse {
    pub key: String,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Invoices
// ---------------------------------------------------------------------------

/// Status of an invoice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InvoiceStatus {
    Pending,
    Settled,
    Expired,
    #[serde(other)]
    Unknown,
}

/// Parameters for creating a new invoice.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

impl CreateInvoiceRequest {
    /// Creates a new invoice request for the given amount in sats.
    pub fn new(amount: i64) -> Self {
        Self {
            amount,
            reference: None,
            memo: None,
        }
    }

    /// Sets the memo (description) for the invoice.
    #[must_use]
    pub fn memo(mut self, memo: impl Into<String>) -> Self {
        self.memo = Some(memo.into());
        self
    }

    /// Sets an external reference for the invoice.
    #[must_use]
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }
}

/// An invoice returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct InvoiceResponse {
    pub number: i32,
    pub status: InvoiceStatus,
    pub amount: i64,
    pub bolt11: String,
    pub reference: Option<String>,
    pub memo: Option<String>,
    pub tx_number: Option<i32>,
    pub created_at: Option<String>,
    pub settled_at: Option<String>,
    pub expires_at: Option<String>,
}

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub limit: Option<i32>,
    pub after: Option<i32>,
}

impl ListParams {
    /// Sets the maximum number of results to return.
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the cursor for pagination (return results after this number).
    #[must_use]
    pub fn after(mut self, after: i32) -> Self {
        self.after = Some(after);
        self
    }
}

/// Type of an invoice SSE event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvoiceEventType {
    Settled,
    Expired,
    Unknown(String),
}

impl From<&str> for InvoiceEventType {
    fn from(s: &str) -> Self {
        match s {
            "settled" => Self::Settled,
            "expired" => Self::Expired,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// A real-time invoice event from the SSE stream.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InvoiceEvent {
    pub event: InvoiceEventType,
    pub data: InvoiceResponse,
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

/// Status of a payment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PaymentStatus {
    Pending,
    Processing,
    Settled,
    Failed,
    #[serde(other)]
    Unknown,
}

/// Parameters for creating a new payment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentRequest {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

impl CreatePaymentRequest {
    /// Creates a new payment request to the given target
    /// (bolt11 invoice, Lightning address, or LN URL).
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            amount: None,
            idempotency_key: None,
            max_fee: None,
            reference: None,
        }
    }

    /// Sets the amount in sats (required for Lightning address / LN URL payments).
    #[must_use]
    pub fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Sets an idempotency key to prevent duplicate payments.
    #[must_use]
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Sets the maximum routing fee in sats.
    #[must_use]
    pub fn max_fee(mut self, fee: i64) -> Self {
        self.max_fee = Some(fee);
        self
    }

    /// Sets an external reference for the payment.
    #[must_use]
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }
}

/// A payment returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PaymentResponse {
    pub number: i32,
    pub status: PaymentStatus,
    pub amount: i64,
    pub max_fee: i64,
    pub actual_fee: Option<i64>,
    pub address: String,
    pub reference: Option<String>,
    pub tx_number: Option<i32>,
    pub failure_reason: Option<String>,
    pub created_at: Option<String>,
    pub settled_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Addresses
// ---------------------------------------------------------------------------

/// Parameters for creating a new Lightning address.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateAddressRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// A Lightning address returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AddressResponse {
    pub address: String,
    pub generated: bool,
    pub cost: i64,
    pub created_at: Option<String>,
}

/// Parameters for transferring a Lightning address to another wallet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferAddressRequest {
    pub target_wallet_key: String,
}

impl TransferAddressRequest {
    pub fn new(target_wallet_key: impl Into<String>) -> Self {
        Self {
            target_wallet_key: target_wallet_key.into(),
        }
    }
}

/// Response from transferring a Lightning address.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransferAddressResponse {
    pub address: String,
    pub transferred_to: String,
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

/// Type of a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TransactionType {
    Credit,
    Debit,
    #[serde(other)]
    Unknown,
}

/// A transaction returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransactionResponse {
    pub number: i32,
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub amount: i64,
    pub balance_after: i64,
    pub network_fee: i64,
    pub service_fee: i64,
    pub payment_hash: Option<String>,
    pub preimage: Option<String>,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

/// Parameters for creating a new webhook.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    pub url: String,
}

impl CreateWebhookRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// Response from creating a new webhook, including the signing secret.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateWebhookResponse {
    pub id: String,
    pub url: String,
    pub secret: String,
    pub created_at: Option<String>,
}

/// A webhook returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookResponse {
    pub id: String,
    pub url: String,
    pub active: bool,
    pub created_at: Option<String>,
}

// ---------------------------------------------------------------------------
// Backup / Restore
// ---------------------------------------------------------------------------

/// Response containing the wallet's recovery passphrase.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RecoveryBackupResponse {
    pub passphrase: String,
}

/// Parameters for restoring a wallet from a recovery passphrase.
#[derive(Debug, Clone, Serialize)]
pub struct RecoveryRestoreRequest {
    pub passphrase: String,
}

impl RecoveryRestoreRequest {
    pub fn new(passphrase: impl Into<String>) -> Self {
        Self {
            passphrase: passphrase.into(),
        }
    }
}

/// Response from a wallet recovery.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RecoveryRestoreResponse {
    pub wallet_id: String,
    pub name: String,
    pub primary_key: String,
    pub secondary_key: String,
}

/// Response from beginning a passkey backup flow.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BackupPasskeyBeginResponse {
    pub session_id: String,
    pub options: HashMap<String, serde_json::Value>,
}

/// Parameters for completing a passkey backup flow.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPasskeyCompleteRequest {
    pub session_id: String,
    pub attestation: HashMap<String, serde_json::Value>,
}

/// Response from beginning a passkey restore flow.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RestorePasskeyBeginResponse {
    pub session_id: String,
    pub options: HashMap<String, serde_json::Value>,
}

/// Parameters for completing a passkey restore flow.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePasskeyCompleteRequest {
    pub session_id: String,
    pub assertion: HashMap<String, serde_json::Value>,
}

/// Response from completing a passkey restore flow.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RestorePasskeyCompleteResponse {
    pub wallet_id: String,
    pub name: String,
    pub primary_key: String,
    pub secondary_key: String,
}
