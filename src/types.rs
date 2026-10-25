//! Request and response types for the LnBot API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

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
// Service status
// ---------------------------------------------------------------------------

/// State of ln.bot as a whole, or of one of its parts.
///
/// More states may be added. One this SDK doesn't know decodes as
/// [`Other`](ServiceState::Other) with the value as sent, so it never fails
/// to parse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
pub enum ServiceState {
    /// Working normally.
    Operational,
    /// Working, but slower or failing more than usual.
    Degraded,
    /// Not working.
    Outage,
    /// Not measured for 5 minutes: the numbers are as old as `updated_at`.
    Unknown,
    /// A state this version of the SDK doesn't know, as sent by the API.
    Other(String),
}

impl ServiceState {
    /// The value as the API sends it, such as `"operational"`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Operational => "operational",
            Self::Degraded => "degraded",
            Self::Outage => "outage",
            Self::Unknown => "unknown",
            Self::Other(s) => s,
        }
    }
}

impl From<&str> for ServiceState {
    fn from(s: &str) -> Self {
        match s {
            "operational" => Self::Operational,
            "degraded" => Self::Degraded,
            "outage" => Self::Outage,
            "unknown" => Self::Unknown,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<String> for ServiceState {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<ServiceState> for String {
    fn from(state: ServiceState) -> Self {
        match state {
            ServiceState::Other(s) => s,
            known => known.as_str().to_string(),
        }
    }
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Level of a notice from the operators.
///
/// More levels may be added. One this SDK doesn't know decodes as
/// [`Other`](NoticeLevel::Other) with the value as sent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
pub enum NoticeLevel {
    /// For information: no effect on the status.
    Info,
    /// The overall status is at least `degraded`.
    Warning,
    /// The overall status is `outage`.
    Outage,
    /// A level this version of the SDK doesn't know, as sent by the API.
    Other(String),
}

impl NoticeLevel {
    /// The value as the API sends it, such as `"warning"`.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Outage => "outage",
            Self::Other(s) => s,
        }
    }
}

impl From<&str> for NoticeLevel {
    fn from(s: &str) -> Self {
        match s {
            "info" => Self::Info,
            "warning" => Self::Warning,
            "outage" => Self::Outage,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<String> for NoticeLevel {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<NoticeLevel> for String {
    fn from(level: NoticeLevel) -> Self {
        match level {
            NoticeLevel::Other(s) => s,
            known => known.as_str().to_string(),
        }
    }
}

impl fmt::Display for NoticeLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The service right now, from [`LnBot::status`](crate::LnBot::status).
///
/// Measured about once a minute.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ServiceStatus {
    /// Overall state.
    pub status: ServiceState,
    /// When it was measured (ISO 8601). `None` if it never was.
    pub updated_at: Option<String>,
    /// A message from the operators. `None` when there is none.
    pub notice: Option<StatusNotice>,
    /// Each part of the service.
    pub components: StatusComponents,
}

/// A message from the operators.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusNotice {
    pub level: NoticeLevel,
    /// Up to 500 characters, to show as is.
    pub message: String,
    /// When it was posted (ISO 8601).
    pub since: String,
    /// When it came down (ISO 8601). `None` while it is up, so always `None`
    /// in [`ServiceStatus`].
    pub until: Option<String>,
}

/// The parts of the service.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusComponents {
    /// Lightning payments out.
    pub sending: SendingStatus,
    /// Payments in.
    pub receiving: ReceivingStatus,
    /// The Lightning node.
    pub node: NodeStatus,
    /// Webhook delivery.
    pub webhooks: WebhooksStatus,
}

/// Sending right now.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SendingStatus {
    pub status: ServiceState,
    /// Payments being sent now.
    pub in_progress: i64,
    /// Those payments by [stage](PaymentStage), keyed by the stage as the API
    /// sends it (`queued`, `retrying`, `sending`, `in_flight`, `confirming`,
    /// `on_hold`, or one added later). Stages with none are left out, so it
    /// can be empty.
    pub by_stage: HashMap<String, i64>,
    /// How long the oldest of them has waited. `None` when there are none.
    pub longest_wait_seconds: Option<f64>,
    /// Payments sent in the last hour.
    pub sent_last_hour: i64,
    /// Payments failed in the last hour, for any reason.
    pub failed_last_hour: i64,
    /// Of those failures, the ones the service caused. The rest were the
    /// invoice's or the route's.
    pub service_failures_last_hour: i64,
    /// Median time from request to settlement. `None` when nothing was sent.
    pub median_seconds: Option<f64>,
    /// 95th percentile time from request to settlement. `None` when nothing
    /// was sent.
    pub p95_seconds: Option<f64>,
}

/// Receiving right now.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReceivingStatus {
    pub status: ServiceState,
    /// Incoming payments credited in the last hour.
    pub credited_last_hour: i64,
    /// Incoming payments waiting to be credited now.
    pub credit_backlog: Option<i64>,
}

/// The Lightning node right now.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NodeStatus {
    pub status: ServiceState,
    /// How long the node took to answer. `None` when it didn't answer.
    pub latency_ms: Option<i64>,
}

/// Webhook delivery right now.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhooksStatus {
    pub status: ServiceState,
    /// Deliveries waiting to go out.
    pub backlog: Option<i64>,
}

/// The service over time, from
/// [`LnBot::status_history`](crate::LnBot::status_history).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ServiceStatusHistory {
    /// The range, as the API sends it: `24h`, `7d`, `30d` or `90d`.
    pub range: String,
    /// Length of one bucket: 300, 3600, 21600 or 86400 seconds.
    pub bucket_seconds: i64,
    /// Start of the first bucket (ISO 8601).
    pub from: String,
    /// When the history was read (ISO 8601).
    pub to: String,
    /// The whole range.
    pub summary: StatusSummary,
    /// Oldest first, aligned to UTC. The last one is still filling, so there
    /// is one more than a full range holds: don't assume the count.
    pub buckets: Vec<StatusBucket>,
    /// The operators' notices that were up during the range, oldest first.
    pub notices: Vec<StatusNotice>,
}

/// Totals over the whole range of a [`ServiceStatusHistory`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusSummary {
    /// Measurements in the range, about one a minute.
    pub samples: i64,
    /// Percent of measured time in each state, per part.
    pub uptime: StatusUptime,
    pub sent: i64,
    pub failed: i64,
    /// Of the failures, the ones the service caused.
    pub service_failures: i64,
    /// Incoming payments credited.
    pub credited: i64,
    /// Approximate median settlement time, from `settle_times`. `None` when
    /// nothing was sent.
    pub median_seconds: Option<f64>,
    /// Approximate 95th percentile settlement time, from `settle_times`.
    /// `None` when nothing was sent.
    pub p95_seconds: Option<f64>,
    /// Settlement times as a histogram.
    pub settle_times: Vec<SettleTime>,
    /// Average node response time.
    pub avg_latency_ms: Option<f64>,
    /// Slowest node response time.
    pub max_latency_ms: Option<i64>,
}

/// Uptime over the range, per part and overall.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusUptime {
    pub overall: UptimeShare,
    pub sending: UptimeShare,
    pub receiving: UptimeShare,
    pub node: UptimeShare,
    pub webhooks: UptimeShare,
}

/// Percent of measured time spent in each state. Each is `None` when nothing
/// was measured.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UptimeShare {
    pub operational: Option<f64>,
    pub degraded: Option<f64>,
    pub outage: Option<f64>,
}

/// One bar of the settlement time histogram.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SettleTime {
    /// Upper bound: 1, 2, 3, 5, 10, 20, 30, 60, 120, 300 or 600 seconds.
    /// `None` on the last bar, which holds anything slower.
    pub up_to_seconds: Option<f64>,
    /// Payments that settled within this bound and above the previous one.
    pub count: i64,
}

/// One bucket of a [`ServiceStatusHistory`].
///
/// `degraded` and `outage` count the bucket's measurements that found that
/// part in that state; the rest found it operational. A bucket's uptime is
/// `(samples - degraded - outage) / samples`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StatusBucket {
    /// Start of the bucket (ISO 8601).
    pub at: String,
    /// Measurements in the bucket, about one a minute. `0` means it wasn't
    /// measured, and its percentiles and gauges are `None`: treat it as no
    /// data, not as up or down.
    pub samples: i64,
    /// ln.bot as a whole.
    pub overall: BucketState,
    pub sending: SendingBucket,
    pub receiving: ReceivingBucket,
    pub node: NodeBucket,
    pub webhooks: WebhooksBucket,
}

/// How many of a bucket's measurements found ln.bot degraded or in an outage.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BucketState {
    pub degraded: i64,
    pub outage: i64,
}

/// Sending during one bucket.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SendingBucket {
    pub degraded: i64,
    pub outage: i64,
    pub sent: i64,
    pub failed: i64,
    /// Of the failures, the ones the service caused.
    pub service_failures: i64,
    /// `None` when nothing was sent.
    pub median_seconds: Option<f64>,
    /// `None` when nothing was sent.
    pub p95_seconds: Option<f64>,
    /// Most payments in progress at once.
    pub max_in_progress: Option<i64>,
    /// Longest wait of a payment in progress.
    pub max_wait_seconds: Option<f64>,
}

/// Receiving during one bucket.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ReceivingBucket {
    pub degraded: i64,
    pub outage: i64,
    /// Incoming payments credited.
    pub credited: i64,
    /// Most incoming payments waiting to be credited at once.
    pub max_backlog: Option<i64>,
}

/// The Lightning node during one bucket.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NodeBucket {
    pub degraded: i64,
    pub outage: i64,
    /// Average response time.
    pub avg_latency_ms: Option<f64>,
    /// Slowest response time.
    pub max_latency_ms: Option<i64>,
}

/// Webhook delivery during one bucket.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhooksBucket {
    pub degraded: i64,
    pub outage: i64,
    /// Most deliveries waiting at once.
    pub max_backlog: Option<i64>,
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
    /// The wait ran out before the invoice settled or expired. `data` is the
    /// invoice's current state: read it again or watch again.
    Timeout,
    Unknown(String),
}

impl From<&str> for InvoiceEventType {
    fn from(s: &str) -> Self {
        match s {
            "settled" => Self::Settled,
            "expired" => Self::Expired,
            "timeout" => Self::Timeout,
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
///
/// A payment can go from `Processing` back to `Pending` when it is retried:
/// treat both as in progress. [`PaymentStage`] says more precisely where it is.
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

/// Where a payment is, more precisely than its [`PaymentStatus`].
///
/// | Stage | With status |
/// |---|---|
/// | `Queued` | `Pending` |
/// | `Retrying` | `Pending` |
/// | `Sending` | `Pending` or `Processing` |
/// | `InFlight` | `Processing` |
/// | `Confirming` | `Processing` |
/// | `OnHold` | `Processing` |
/// | `Settled` | `Settled` |
/// | `Failed` | `Failed` |
///
/// More stages may be added. One this SDK doesn't know decodes as
/// [`Unknown`](PaymentStage::Unknown): go by the payment's `status` then.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PaymentStage {
    /// Waiting to be sent.
    Queued,
    /// Not sent yet and retried for a few minutes, until
    /// [`retry_until`](PaymentResponse::retry_until). If it still can't be
    /// sent, it fails and the funds are released.
    Retrying,
    /// Being sent, or between ln.bot wallets, being transferred to the
    /// recipient's wallet.
    Sending,
    /// Sent: the Lightning network is completing it, usually within seconds.
    InFlight,
    /// Checking with the Lightning network whether it went out. The funds
    /// stay on hold until then.
    Confirming,
    /// Interrupted by an earlier version of the service and on hold until it
    /// is reviewed. The funds are held, not lost.
    OnHold,
    Settled,
    Failed,
    /// A stage this version of the SDK doesn't know.
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
    /// Maximum network fee in sats: 0.25% plus Lightning routing.
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
    /// Network fee actually charged in sats (0.25% plus routing).
    /// `None` until the payment settles.
    pub actual_fee: Option<i64>,
    pub address: String,
    pub reference: Option<String>,
    pub preimage: Option<String>,
    pub tx_number: Option<i32>,
    pub failure_reason: Option<String>,
    pub created_at: Option<String>,
    pub settled_at: Option<String>,
    /// Where the payment is, more precisely than `status`. `None` from an API
    /// that doesn't send it. A stage this SDK doesn't know is
    /// [`PaymentStage::Unknown`]: go by `status` then.
    #[serde(default)]
    pub stage: Option<PaymentStage>,
    /// A sentence to show the user as is: what is happening while the payment
    /// is in progress, the `failure_reason` once it failed, `None` once it
    /// settled.
    #[serde(default)]
    pub status_detail: Option<String>,
    /// Only with [`PaymentStage::Retrying`]: until when the payment is
    /// retried (ISO 8601). If it still can't be sent by then, it fails within
    /// about a minute and the funds are released. `None` otherwise.
    #[serde(default)]
    pub retry_until: Option<String>,
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
    /// Estimated network fee in sats (0.25% plus routing). `None`
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
    /// The wait ran out before the payment settled or failed. `data` is the
    /// payment's current state, typically `pending` or `processing`: read it
    /// again or watch again.
    Timeout,
    Unknown(String),
}

impl From<&str> for PaymentEventType {
    fn from(s: &str) -> Self {
        match s {
            "settled" => Self::Settled,
            "failed" => Self::Failed,
            "timeout" => Self::Timeout,
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
///
/// `event` is `invoice.created`, `invoice.settled`, `payment.created`,
/// `payment.settled` or `payment.failed`. `data` holds the invoice or payment
/// with the same camelCase keys as REST responses (the SDK converts the
/// snake_case keys sent on the wire), so it decodes with
/// `serde_json::from_value` into [`InvoiceResponse`] or [`PaymentResponse`].
/// The `*.created` events can leave out fields that are only set later,
/// which then decode as `None`.
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
    /// Maximum network fee in sats (0.25% plus Lightning routing).
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
    /// Network fee actually charged in sats (0.25% plus routing).
    /// `None` until the payment settles.
    pub fee: Option<i64>,
    pub payment_number: i32,
    pub status: String,
}
