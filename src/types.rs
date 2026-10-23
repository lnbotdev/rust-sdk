//! Request and response types for the LnBot API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

/// Response from registering a new account.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RegisterResponse {
    pub user_id: String,
    pub primary_key: String,
    pub secondary_key: String,
    pub recovery_passphrase: String,
}

/// Response from the `/v1/me` identity endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MeResponse {
    pub user_id: String,
    pub key_name: Option<String>,
    pub wallet_id: Option<String>,
}

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

/// Response from creating a new wallet.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateWalletResponse {
    pub wallet_id: String,
    pub name: String,
    pub address: String,
}

/// A wallet in the list returned by `/v1/wallets`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WalletListItem {
    pub wallet_id: String,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Wallet Key
// ---------------------------------------------------------------------------

/// Response from creating or rotating a wallet key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WalletKeyResponse {
    pub key: String,
}

/// Response from getting wallet key info.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WalletKeyInfoResponse {
    pub hint: String,
    pub created_at: Option<String>,
    pub last_used_at: Option<String>,
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

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
    pub preimage: Option<String>,
    pub tx_number: Option<i32>,
    pub created_at: Option<String>,
    pub settled_at: Option<String>,
    pub expires_at: Option<String>,
}

/// Parameters for creating an invoice for a specific wallet (unauthenticated).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceForWalletRequest {
    pub wallet_id: String,
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl CreateInvoiceForWalletRequest {
    pub fn new(wallet_id: impl Into<String>, amount: i64) -> Self {
        Self {
            wallet_id: wallet_id.into(),
            amount,
            reference: None,
            comment: None,
        }
    }

    #[must_use]
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Parameters for creating an invoice for a Lightning address (unauthenticated).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceForAddressRequest {
    pub address: String,
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl CreateInvoiceForAddressRequest {
    pub fn new(address: impl Into<String>, amount: i64) -> Self {
        Self {
            address: address.into(),
            amount,
            tag: None,
            comment: None,
        }
    }

    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// An invoice created via wallet ID or Lightning address.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AddressInvoiceResponse {
    pub bolt11: String,
    pub amount: i64,
    pub expires_at: Option<String>,
}

/// Pagination parameters for list endpoints.
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub limit: Option<i32>,
    pub after: Option<i32>,
}

impl ListParams {
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

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
    /// Maximum network fee in sats: Spark's fee (0.25%) plus Lightning routing.
    ///
    /// Leave this unset unless you need a different cap. The server then uses
    /// its default of 1% of the amount, minimum 20 sats. When set, it must be
    /// between 1 and the larger of 10,000 sats and 5% of the amount. The fee is
    /// quoted when the payment is created: a quote above the cap rejects the
    /// payment with a 400, and only the quote plus a small buffer is held while
    /// the payment is in flight. Ignored for internal transfers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

impl CreatePaymentRequest {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            amount: None,
            idempotency_key: None,
            max_fee: None,
            reference: None,
        }
    }

    #[must_use]
    pub fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    #[must_use]
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Sets the maximum network fee in sats. Optional: see
    /// [`max_fee`](CreatePaymentRequest::max_fee) for the default and limits.
    #[must_use]
    pub fn max_fee(mut self, fee: i64) -> Self {
        self.max_fee = Some(fee);
        self
    }

    #[must_use]
    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }
}

/// A payment returned by the API.
///
/// When the payment settles, the wallet is debited
/// `amount + actual_fee + service_fee`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PaymentResponse {
    pub number: i32,
    pub status: PaymentStatus,
    pub amount: i64,
    /// Network fee reserved for this payment in sats: the fee quote plus a
    /// small buffer, or the cap when no quote was available. The unused part
    /// is released at settlement.
    pub max_fee: i64,
    /// Service fee in sats: 0.25%, minimum 1 sat. 0 for internal transfers.
    pub service_fee: i64,
    /// Network fee actually charged in sats (Spark's fee plus routing).
    /// `None` until the payment settles.
    pub actual_fee: Option<i64>,
    pub address: String,
    pub reference: Option<String>,
    pub preimage: Option<String>,
    pub tx_number: Option<i32>,
    pub failure_reason: Option<String>,
    pub created_at: Option<String>,
    pub settled_at: Option<String>,
}

/// Response from resolving and pricing a payment target.
///
/// The fee fields are only filled in when there is an amount to price: the
/// invoice amount for a BOLT11 invoice, or the amount passed to
/// [`resolve_with_amount`](crate::resources::PaymentsResource::resolve_with_amount).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResolveTargetResponse {
    /// Not returned by the API. Always empty.
    #[deprecated(since = "1.1.0", note = "not returned by the API; always empty")]
    #[serde(default)]
    pub target: String,
    /// Target type: `lightning_address`, `lnurl` or `bolt11`.
    #[serde(rename = "type")]
    pub target_type: String,
    /// Minimum accepted amount in sats. `None` for BOLT11 invoices.
    pub min: Option<i64>,
    /// Maximum accepted amount in sats. `None` for BOLT11 invoices.
    pub max: Option<i64>,
    /// True if the recipient expects a fixed amount.
    pub fixed: bool,
    /// The amount priced in sats: the invoice amount for a BOLT11 invoice,
    /// otherwise the amount passed. `None` when there is no amount.
    pub amount: Option<i64>,
    /// True when the recipient is a wallet on ln.bot: a free internal transfer.
    pub internal: bool,
    /// Estimated network fee in sats (Spark's 0.25% plus routing). `None`
    /// without an amount or when no estimate is available. 0 for internal
    /// transfers.
    pub network_fee_estimate: Option<i64>,
    /// Service fee in sats: 0.25%, minimum 1 sat. 0 for internal transfers.
    /// `None` without an amount.
    pub service_fee: Option<i64>,
    /// Network fee that would be held if the payment is created without
    /// `max_fee`. `None` without an amount, or when `fee_warning` is set.
    pub max_fee: Option<i64>,
    /// Expected total cost in sats: `amount + network_fee_estimate + service_fee`.
    /// `None` without an amount.
    pub total: Option<i64>,
    /// For a Lightning address or LNURL resolved without an amount: the most
    /// this wallet can send there now, after fees. 0 if the wallet can't cover
    /// the recipient's minimum. `None` otherwise.
    pub max_sendable: Option<i64>,
    /// Set when creating the payment without `max_fee` would be rejected
    /// because the fee is unusually high. The response is still a 200.
    pub fee_warning: Option<String>,
    /// Not returned by the API. Always `None`.
    #[deprecated(since = "1.1.0", note = "not returned by the API; always None")]
    #[serde(default)]
    pub description: Option<String>,
}

/// Type of a payment SSE event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaymentEventType {
    Settled,
    Failed,
    Unknown(String),
}

impl From<&str> for PaymentEventType {
    fn from(s: &str) -> Self {
        match s {
            "settled" => Self::Settled,
            "failed" => Self::Failed,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// A real-time payment event from the SSE stream.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PaymentEvent {
    pub event: PaymentEventType,
    pub data: PaymentResponse,
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
// Wallet event stream
// ---------------------------------------------------------------------------

/// A real-time event from the wallet event stream.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WalletEvent {
    pub event: String,
    pub created_at: String,
    pub data: serde_json::Value,
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

// ---------------------------------------------------------------------------
// L402
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateL402ChallengeRequest {
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caveats: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L402ChallengeResponse {
    pub macaroon: String,
    pub invoice: String,
    pub payment_hash: String,
    pub expires_at: String,
    pub www_authenticate: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyL402Request {
    pub authorization: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyL402Response {
    pub valid: bool,
    pub payment_hash: Option<String>,
    pub caveats: Option<Vec<String>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PayL402Request {
    pub www_authenticate: String,
    /// Maximum network fee in sats (Spark's fee plus Lightning routing).
    /// Leave it `None` to use the server default of 1% of the amount, minimum
    /// 20 sats. Same meaning and limits as
    /// [`CreatePaymentRequest::max_fee`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L402PayResponse {
    pub authorization: Option<String>,
    pub payment_hash: String,
    pub preimage: Option<String>,
    pub amount: i64,
    /// Network fee actually charged in sats (Spark's fee plus routing).
    /// `None` until the payment settles.
    pub fee: Option<i64>,
    pub payment_number: i32,
    pub status: String,
}
