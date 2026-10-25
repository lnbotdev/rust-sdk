use lnbot::*;

// ---------------------------------------------------------------------------
// Request builders
// ---------------------------------------------------------------------------

#[test]
fn create_invoice_request_defaults() {
    let req = CreateInvoiceRequest::new(100);
    assert_eq!(req.amount, 100);
    assert!(req.memo.is_none());
    assert!(req.reference.is_none());
}

#[test]
fn create_invoice_request_with_options() {
    let req = CreateInvoiceRequest::new(500)
        .memo("coffee")
        .reference("order-42");
    assert_eq!(req.amount, 500);
    assert_eq!(req.memo.as_deref(), Some("coffee"));
    assert_eq!(req.reference.as_deref(), Some("order-42"));
}

#[test]
fn create_payment_request_builder() {
    let req = CreatePaymentRequest::new("lnbc1...")
        .amount(50)
        .max_fee(10)
        .idempotency_key("idem-1")
        .reference("pay-ref");
    assert_eq!(req.target, "lnbc1...");
    assert_eq!(req.amount, Some(50));
    assert_eq!(req.max_fee, Some(10));
    assert_eq!(req.idempotency_key.as_deref(), Some("idem-1"));
    assert_eq!(req.reference.as_deref(), Some("pay-ref"));
}

#[test]
fn create_payment_request_defaults() {
    let req = CreatePaymentRequest::new("target");
    assert!(req.amount.is_none());
    assert!(req.max_fee.is_none());
    assert!(req.idempotency_key.is_none());
    assert!(req.reference.is_none());
}

#[test]
fn update_wallet_request_new() {
    let req = UpdateWalletRequest::new("Renamed");
    assert_eq!(req.name, "Renamed");
}

#[test]
fn create_address_request_default() {
    let req = CreateAddressRequest::default();
    assert!(req.address.is_none());
}

#[test]
fn transfer_address_request_new() {
    let req = TransferAddressRequest::new("key_target");
    assert_eq!(req.target_wallet_key, "key_target");
}

#[test]
fn create_webhook_request_new() {
    let req = CreateWebhookRequest::new("https://example.com/hook");
    assert_eq!(req.url, "https://example.com/hook");
}

#[test]
fn recovery_restore_request_new() {
    let req = RecoveryRestoreRequest::new("word1 word2 word3");
    assert_eq!(req.passphrase, "word1 word2 word3");
}

#[test]
fn create_invoice_for_wallet_request_builder() {
    let req = CreateInvoiceForWalletRequest::new("wid", 200)
        .reference("ref")
        .comment("hello");
    assert_eq!(req.wallet_id, "wid");
    assert_eq!(req.amount, 200);
    assert_eq!(req.reference.as_deref(), Some("ref"));
    assert_eq!(req.comment.as_deref(), Some("hello"));
}

#[test]
fn create_invoice_for_address_request_builder() {
    let req = CreateInvoiceForAddressRequest::new("user@ln.bot", 300)
        .tag("payRequest")
        .comment("tip");
    assert_eq!(req.address, "user@ln.bot");
    assert_eq!(req.amount, 300);
    assert_eq!(req.tag.as_deref(), Some("payRequest"));
    assert_eq!(req.comment.as_deref(), Some("tip"));
}

#[test]
fn list_params_default_is_empty() {
    let p = ListParams::default();
    assert!(p.limit.is_none());
    assert!(p.after.is_none());
}

#[test]
fn list_params_builder() {
    let p = ListParams::default().limit(10).after(5);
    assert_eq!(p.limit, Some(10));
    assert_eq!(p.after, Some(5));
}

// ---------------------------------------------------------------------------
// Serialization (camelCase, skip_serializing_if)
// ---------------------------------------------------------------------------

#[test]
fn create_invoice_serializes_camel_case() {
    let req = CreateInvoiceRequest::new(100).reference("ref-1");
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["amount"], 100);
    assert_eq!(json["reference"], "ref-1");
    assert!(json.get("memo").is_none());
}

#[test]
fn create_payment_serializes_camel_case() {
    let req = CreatePaymentRequest::new("target").amount(100).max_fee(5);
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["target"], "target");
    assert_eq!(json["amount"], 100);
    assert_eq!(json["maxFee"], 5);
    assert!(json.get("idempotencyKey").is_none());
    assert!(json.get("reference").is_none());
}

#[test]
fn transfer_address_serializes_camel_case() {
    let req = TransferAddressRequest::new("key_abc");
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["targetWalletKey"], "key_abc");
}

#[test]
fn pay_l402_serializes_camel_case() {
    let req = PayL402Request {
        www_authenticate: "L402 mac:inv".into(),
        max_fee: Some(10),
        reference: None,
        wait: Some(true),
        timeout: Some(60),
    };
    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["wwwAuthenticate"], "L402 mac:inv");
    assert_eq!(json["maxFee"], 10);
    assert_eq!(json["wait"], true);
    assert_eq!(json["timeout"], 60);
    assert!(json.get("reference").is_none());
}

// ---------------------------------------------------------------------------
// Deserialization (camelCase, #[serde(other)])
// ---------------------------------------------------------------------------

#[test]
fn wallet_response_deserializes_camel_case() {
    let json = r#"{"walletId":"w1","name":"My Wallet","balance":1000,"onHold":100,"available":900}"#;
    let w: WalletResponse = serde_json::from_str(json).unwrap();
    assert_eq!(w.wallet_id, "w1");
    assert_eq!(w.name, "My Wallet");
    assert_eq!(w.balance, 1000);
    assert_eq!(w.on_hold, 100);
    assert_eq!(w.available, 900);
}

#[test]
fn create_wallet_response_deserializes() {
    let json = r#"{"walletId":"wal_1","name":"Bot","address":"a@ln.bot"}"#;
    let w: CreateWalletResponse = serde_json::from_str(json).unwrap();
    assert_eq!(w.wallet_id, "wal_1");
    assert_eq!(w.name, "Bot");
    assert_eq!(w.address, "a@ln.bot");
}

#[test]
fn wallet_list_item_deserializes() {
    let json = r#"{"walletId":"wal_1","name":"My Wallet"}"#;
    let w: WalletListItem = serde_json::from_str(json).unwrap();
    assert_eq!(w.wallet_id, "wal_1");
    assert_eq!(w.name, "My Wallet");
}

#[test]
fn register_response_deserializes() {
    let json = r#"{"userId":"u1","primaryKey":"uk_1","secondaryKey":"uk_2","recoveryPassphrase":"words"}"#;
    let r: RegisterResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.user_id, "u1");
    assert_eq!(r.primary_key, "uk_1");
    assert_eq!(r.recovery_passphrase, "words");
}

#[test]
fn me_response_deserializes() {
    let json = r#"{"userId":"u1","keyName":"primary","walletId":null}"#;
    let m: MeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(m.user_id, "u1");
    assert_eq!(m.key_name.as_deref(), Some("primary"));
    assert!(m.wallet_id.is_none());
}

#[test]
fn me_response_minimal() {
    let json = r#"{"userId":"u1"}"#;
    let m: MeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(m.user_id, "u1");
    assert!(m.key_name.is_none());
}

#[test]
fn wallet_key_response_deserializes() {
    let json = r#"{"key":"wk_abc"}"#;
    let r: WalletKeyResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.key, "wk_abc");
}

#[test]
fn wallet_key_info_response_deserializes() {
    let json = r#"{"hint":"wk_abc...xyz","createdAt":"2024-01-01T00:00:00Z","lastUsedAt":null}"#;
    let r: WalletKeyInfoResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.hint, "wk_abc...xyz");
    assert_eq!(r.created_at.as_deref(), Some("2024-01-01T00:00:00Z"));
    assert!(r.last_used_at.is_none());
}

#[test]
fn resolve_target_response_deserializes_all_fields() {
    let json = r#"{
        "type":"bolt11","min":null,"max":null,"fixed":true,"amount":250000,
        "internal":false,"networkFeeEstimate":700,"serviceFee":625,"maxFee":805,
        "total":251325,"maxSendable":null,"feeWarning":null
    }"#;
    let r: ResolveTargetResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.target_type, "bolt11");
    assert!(r.min.is_none());
    assert!(r.max.is_none());
    assert!(r.fixed);
    assert_eq!(r.amount, Some(250000));
    assert!(!r.internal);
    assert_eq!(r.network_fee_estimate, Some(700));
    assert_eq!(r.service_fee, Some(625));
    assert_eq!(r.max_fee, Some(805));
    assert_eq!(r.total, Some(251325));
    assert!(r.max_sendable.is_none());
    assert!(r.fee_warning.is_none());
}

#[test]
fn resolve_target_response_lightning_address_without_amount() {
    let json = r#"{
        "type":"lightning_address","min":1,"max":1000000000,"fixed":false,"amount":null,
        "internal":false,"networkFeeEstimate":null,"serviceFee":null,"maxFee":null,
        "total":null,"maxSendable":49750,"feeWarning":null
    }"#;
    let r: ResolveTargetResponse = serde_json::from_str(json).unwrap();
    assert_eq!(r.target_type, "lightning_address");
    assert_eq!(r.min, Some(1));
    assert_eq!(r.max, Some(1000000000));
    assert!(!r.fixed);
    assert!(r.amount.is_none());
    assert!(r.network_fee_estimate.is_none());
    assert!(r.service_fee.is_none());
    assert!(r.max_fee.is_none());
    assert!(r.total.is_none());
    assert_eq!(r.max_sendable, Some(49750));
}

#[test]
fn resolve_target_response_fee_warning() {
    let json = r#"{
        "type":"bolt11","min":null,"max":null,"fixed":true,"amount":1000,
        "internal":false,"networkFeeEstimate":150,"serviceFee":3,"maxFee":null,
        "total":1153,"maxSendable":null,
        "feeWarning":"Network fee estimate (150 sats) is above the default limit for this amount (20 sats). Set maxFee to allow a higher fee."
    }"#;
    let r: ResolveTargetResponse = serde_json::from_str(json).unwrap();
    assert!(r.max_fee.is_none());
    assert!(r.fee_warning.is_some());
}

#[test]
#[allow(deprecated)]
fn resolve_target_response_legacy_fields_default() {
    let json = r#"{
        "type":"lightning_address","min":1,"max":1000,"fixed":false,"amount":null,
        "internal":true,"networkFeeEstimate":null,"serviceFee":null,"maxFee":null,
        "total":null,"maxSendable":500,"feeWarning":null
    }"#;
    let r: ResolveTargetResponse = serde_json::from_str(json).unwrap();
    assert!(r.internal);
    assert_eq!(r.target, "");
    assert!(r.description.is_none());
}

#[test]
fn invoice_response_optional_fields() {
    let json = r#"{
        "number":1,"status":"pending","amount":100,"bolt11":"lnbc1...",
        "reference":null,"memo":null,"preimage":null,"txNumber":null,
        "createdAt":null,"settledAt":null,"expiresAt":null
    }"#;
    let inv: InvoiceResponse = serde_json::from_str(json).unwrap();
    assert_eq!(inv.number, 1);
    assert_eq!(inv.status, InvoiceStatus::Pending);
    assert!(inv.reference.is_none());
    assert!(inv.preimage.is_none());
}

#[test]
fn payment_response_deserializes() {
    let json = r#"{
        "number":1,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,
        "actualFee":2,"address":"user@ln.bot","reference":null,"preimage":"abc",
        "txNumber":5,"failureReason":null,"createdAt":"2024-01-01T00:00:00Z","settledAt":"2024-01-01T00:00:01Z"
    }"#;
    let p: PaymentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(p.number, 1);
    assert_eq!(p.status, PaymentStatus::Settled);
    assert_eq!(p.actual_fee, Some(2));
    assert_eq!(p.address, "user@ln.bot");
    assert_eq!(p.preimage.as_deref(), Some("abc"));
    assert_eq!(p.tx_number, Some(5));
}

const PAYMENT_RETRYING: &str = r#"{"number":4,"status":"pending","amount":1000,"maxFee":20,"serviceFee":3,"actualFee":null,"address":"lnbc10u1p...","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":"2026-10-25T14:33:53+00:00","settledAt":null,"stage":"retrying","statusDetail":"Not sent yet; retrying for a few minutes. If it still can't be sent, it fails and the funds are released.","retryUntil":"2026-10-25T14:43:53+00:00"}"#;

const PAYMENT_WITHOUT_STAGE: &str = r#"{"number":3,"status":"settled","amount":500,"maxFee":20,"serviceFee":1,"actualFee":2,"address":"x4k7m2@ln.bot","reference":null,"preimage":"9f2c...","txNumber":12,"failureReason":null,"createdAt":"2026-10-25T14:30:00+00:00","settledAt":"2026-10-25T14:30:01+00:00"}"#;

const PAYMENT_UNKNOWN_STAGE: &str = r#"{"number":5,"status":"processing","amount":1000,"maxFee":20,"serviceFee":3,"actualFee":null,"address":"x4k7m2@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":"2026-10-25T14:35:00+00:00","settledAt":null,"stage":"teleporting","statusDetail":"Moving through a new kind of route.","retryUntil":null}"#;

#[test]
fn payment_response_retrying_has_stage_fields() {
    let p: PaymentResponse = serde_json::from_str(PAYMENT_RETRYING).unwrap();
    assert_eq!(p.number, 4);
    assert_eq!(p.status, PaymentStatus::Pending);
    assert_eq!(p.stage, Some(PaymentStage::Retrying));
    assert_eq!(
        p.status_detail.as_deref(),
        Some("Not sent yet; retrying for a few minutes. If it still can't be sent, it fails and the funds are released.")
    );
    assert_eq!(p.retry_until.as_deref(), Some("2026-10-25T14:43:53+00:00"));
}

#[test]
fn payment_response_without_stage_fields() {
    let p: PaymentResponse = serde_json::from_str(PAYMENT_WITHOUT_STAGE).unwrap();
    assert_eq!(p.number, 3);
    assert_eq!(p.status, PaymentStatus::Settled);
    assert_eq!(p.actual_fee, Some(2));
    assert_eq!(p.stage, None);
    assert_eq!(p.status_detail, None);
    assert_eq!(p.retry_until, None);
}

#[test]
fn payment_response_unknown_stage_falls_back() {
    let p: PaymentResponse = serde_json::from_str(PAYMENT_UNKNOWN_STAGE).unwrap();
    assert_eq!(p.stage, Some(PaymentStage::Unknown));
    // The status still says where the payment is.
    assert_eq!(p.status, PaymentStatus::Processing);
    assert_eq!(
        p.status_detail.as_deref(),
        Some("Moving through a new kind of route.")
    );
    assert_eq!(p.retry_until, None);
}

#[test]
fn payment_response_null_stage() {
    let json = r#"{"number":1,"status":"pending","amount":1,"maxFee":20,"serviceFee":1,"actualFee":null,"address":"a","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null,"stage":null,"statusDetail":null,"retryUntil":null}"#;
    let p: PaymentResponse = serde_json::from_str(json).unwrap();
    assert_eq!(p.stage, None);
    assert_eq!(p.status_detail, None);
}

#[test]
fn transaction_response_deserializes_type_field() {
    let json = r#"{
        "number":1,"type":"credit","amount":100,"balanceAfter":100,
        "networkFee":0,"serviceFee":0,"paymentHash":null,"preimage":null,
        "reference":null,"note":null,"createdAt":null
    }"#;
    let tx: TransactionResponse = serde_json::from_str(json).unwrap();
    assert_eq!(tx.tx_type, TransactionType::Credit);
    assert_eq!(tx.balance_after, 100);
}

// ---------------------------------------------------------------------------
// Status enums with #[serde(other)]
// ---------------------------------------------------------------------------

#[test]
fn invoice_status_known_values() {
    assert_eq!(
        serde_json::from_str::<InvoiceStatus>(r#""pending""#).unwrap(),
        InvoiceStatus::Pending,
    );
    assert_eq!(
        serde_json::from_str::<InvoiceStatus>(r#""settled""#).unwrap(),
        InvoiceStatus::Settled,
    );
    assert_eq!(
        serde_json::from_str::<InvoiceStatus>(r#""expired""#).unwrap(),
        InvoiceStatus::Expired,
    );
}

#[test]
fn invoice_status_unknown_falls_back() {
    assert_eq!(
        serde_json::from_str::<InvoiceStatus>(r#""future_status""#).unwrap(),
        InvoiceStatus::Unknown,
    );
}

#[test]
fn payment_status_known_values() {
    assert_eq!(
        serde_json::from_str::<PaymentStatus>(r#""pending""#).unwrap(),
        PaymentStatus::Pending,
    );
    assert_eq!(
        serde_json::from_str::<PaymentStatus>(r#""processing""#).unwrap(),
        PaymentStatus::Processing,
    );
    assert_eq!(
        serde_json::from_str::<PaymentStatus>(r#""settled""#).unwrap(),
        PaymentStatus::Settled,
    );
    assert_eq!(
        serde_json::from_str::<PaymentStatus>(r#""failed""#).unwrap(),
        PaymentStatus::Failed,
    );
}

#[test]
fn payment_status_unknown_falls_back() {
    assert_eq!(
        serde_json::from_str::<PaymentStatus>(r#""future_status""#).unwrap(),
        PaymentStatus::Unknown,
    );
}

#[test]
fn payment_stage_known_values() {
    let cases = [
        ("queued", PaymentStage::Queued),
        ("retrying", PaymentStage::Retrying),
        ("sending", PaymentStage::Sending),
        ("in_flight", PaymentStage::InFlight),
        ("confirming", PaymentStage::Confirming),
        ("on_hold", PaymentStage::OnHold),
        ("settled", PaymentStage::Settled),
        ("failed", PaymentStage::Failed),
    ];
    for (wire, stage) in cases {
        let json = format!("\"{wire}\"");
        assert_eq!(serde_json::from_str::<PaymentStage>(&json).unwrap(), stage);
        assert_eq!(serde_json::to_string(&stage).unwrap(), json);
    }
}

#[test]
fn payment_stage_unknown_falls_back() {
    assert_eq!(
        serde_json::from_str::<PaymentStage>(r#""teleporting""#).unwrap(),
        PaymentStage::Unknown,
    );
}

#[test]
fn transaction_type_unknown_falls_back() {
    assert_eq!(
        serde_json::from_str::<TransactionType>(r#""future_type""#).unwrap(),
        TransactionType::Unknown,
    );
}

// ---------------------------------------------------------------------------
// Service status
// ---------------------------------------------------------------------------

const STATUS_OPERATIONAL: &str = r#"{"status":"operational","updatedAt":"2026-10-25T14:33:53.06+00:00","notice":null,"components":{"sending":{"status":"operational","inProgress":1,"byStage":{"in_flight":1},"longestWaitSeconds":3,"sentLastHour":42,"failedLastHour":1,"serviceFailuresLastHour":0,"medianSeconds":2.1,"p95Seconds":6.8},"receiving":{"status":"operational","creditedLastHour":97,"creditBacklog":0},"node":{"status":"operational","latencyMs":284},"webhooks":{"status":"operational","backlog":0}}}"#;

const STATUS_OUTAGE: &str = r#"{"status":"outage","updatedAt":"2026-10-25T14:33:53.06+00:00","notice":{"level":"outage","message":"Payments are paused while the node recovers.","since":"2026-10-25T14:10:00+00:00","until":null},"components":{"sending":{"status":"outage","inProgress":6,"byStage":{"queued":2,"retrying":4},"longestWaitSeconds":1260,"sentLastHour":3,"failedLastHour":5,"serviceFailuresLastHour":5,"medianSeconds":2.4,"p95Seconds":3.0},"receiving":{"status":"outage","creditedLastHour":4,"creditBacklog":0},"node":{"status":"outage","latencyMs":null},"webhooks":{"status":"operational","backlog":0}}}"#;

const STATUS_STALE: &str = r#"{"status":"unknown","updatedAt":"2026-10-25T13:02:11.00+00:00","notice":null,"components":{"sending":{"status":"unknown","inProgress":0,"byStage":{},"longestWaitSeconds":null,"sentLastHour":12,"failedLastHour":0,"serviceFailuresLastHour":0,"medianSeconds":2.0,"p95Seconds":5.5},"receiving":{"status":"unknown","creditedLastHour":30,"creditBacklog":0},"node":{"status":"unknown","latencyMs":250},"webhooks":{"status":"unknown","backlog":0}}}"#;

const STATUS_NEVER_MEASURED: &str = r#"{"status":"unknown","updatedAt":null,"notice":null,"components":{"sending":{"status":"unknown","inProgress":0,"byStage":{},"longestWaitSeconds":null,"sentLastHour":0,"failedLastHour":0,"serviceFailuresLastHour":0,"medianSeconds":null,"p95Seconds":null},"receiving":{"status":"unknown","creditedLastHour":0,"creditBacklog":null},"node":{"status":"unknown","latencyMs":null},"webhooks":{"status":"unknown","backlog":null}}}"#;

// Saved from the live API on 2026-10-25.
const STATUS_LIVE: &str = include_str!("fixtures/status_live.json");
const HISTORY_24H_LIVE: &str = include_str!("fixtures/status_history_24h.json");
// Cut to three days: one measured, one with a 25-minute outage, one not measured.
const HISTORY_90D: &str = include_str!("fixtures/status_history_90d.json");

#[test]
fn service_state_values() {
    let cases = [
        ("operational", ServiceState::Operational),
        ("degraded", ServiceState::Degraded),
        ("outage", ServiceState::Outage),
        ("unknown", ServiceState::Unknown),
    ];
    for (wire, state) in cases {
        let json = format!("\"{wire}\"");
        assert_eq!(serde_json::from_str::<ServiceState>(&json).unwrap(), state);
        assert_eq!(serde_json::to_string(&state).unwrap(), json);
        assert_eq!(state.as_str(), wire);
        assert_eq!(state.to_string(), wire);
    }
}

#[test]
fn service_state_unknown_value_is_kept() {
    let state: ServiceState = serde_json::from_str(r#""maintenance""#).unwrap();
    assert_eq!(state, ServiceState::Other("maintenance".into()));
    assert_ne!(state, ServiceState::Unknown);
    assert_eq!(state.as_str(), "maintenance");
    assert_eq!(serde_json::to_string(&state).unwrap(), r#""maintenance""#);
}

#[test]
fn notice_level_values() {
    let cases = [
        ("info", NoticeLevel::Info),
        ("warning", NoticeLevel::Warning),
        ("outage", NoticeLevel::Outage),
    ];
    for (wire, level) in cases {
        let json = format!("\"{wire}\"");
        assert_eq!(serde_json::from_str::<NoticeLevel>(&json).unwrap(), level);
        assert_eq!(serde_json::to_string(&level).unwrap(), json);
        assert_eq!(level.to_string(), wire);
    }
    let level: NoticeLevel = serde_json::from_str(r#""critical""#).unwrap();
    assert_eq!(level, NoticeLevel::Other("critical".into()));
    assert_eq!(level.as_str(), "critical");
}

#[test]
fn service_status_operational() {
    let s: ServiceStatus = serde_json::from_str(STATUS_OPERATIONAL).unwrap();
    assert_eq!(s.status, ServiceState::Operational);
    assert_eq!(
        s.updated_at.as_deref(),
        Some("2026-10-25T14:33:53.06+00:00")
    );
    assert!(s.notice.is_none());
    let sending = &s.components.sending;
    assert_eq!(sending.status, ServiceState::Operational);
    assert_eq!(sending.in_progress, 1);
    assert_eq!(sending.by_stage.len(), 1);
    assert_eq!(sending.by_stage["in_flight"], 1);
    assert_eq!(sending.longest_wait_seconds, Some(3.0));
    assert_eq!(sending.sent_last_hour, 42);
    assert_eq!(sending.failed_last_hour, 1);
    assert_eq!(sending.service_failures_last_hour, 0);
    assert_eq!(sending.median_seconds, Some(2.1));
    assert_eq!(sending.p95_seconds, Some(6.8));
    assert_eq!(s.components.receiving.status, ServiceState::Operational);
    assert_eq!(s.components.receiving.credited_last_hour, 97);
    assert_eq!(s.components.receiving.credit_backlog, Some(0));
    assert_eq!(s.components.node.latency_ms, Some(284));
    assert_eq!(s.components.webhooks.backlog, Some(0));
}

#[test]
fn service_status_outage_with_notice() {
    let s: ServiceStatus = serde_json::from_str(STATUS_OUTAGE).unwrap();
    assert_eq!(s.status, ServiceState::Outage);
    let notice = s.notice.as_ref().unwrap();
    assert_eq!(notice.level, NoticeLevel::Outage);
    assert_eq!(
        notice.message,
        "Payments are paused while the node recovers."
    );
    assert_eq!(notice.since, "2026-10-25T14:10:00+00:00");
    assert_eq!(notice.until, None);
    let sending = &s.components.sending;
    assert_eq!(sending.status, ServiceState::Outage);
    assert_eq!(sending.in_progress, 6);
    assert_eq!(sending.by_stage["queued"], 2);
    assert_eq!(sending.by_stage["retrying"], 4);
    assert_eq!(sending.longest_wait_seconds, Some(1260.0));
    assert_eq!(sending.service_failures_last_hour, 5);
    assert_eq!(sending.p95_seconds, Some(3.0));
    assert_eq!(s.components.node.status, ServiceState::Outage);
    assert_eq!(s.components.node.latency_ms, None);
    assert_eq!(s.components.webhooks.status, ServiceState::Operational);
}

#[test]
fn service_status_stale() {
    let s: ServiceStatus = serde_json::from_str(STATUS_STALE).unwrap();
    assert_eq!(s.status, ServiceState::Unknown);
    assert_eq!(
        s.updated_at.as_deref(),
        Some("2026-10-25T13:02:11.00+00:00")
    );
    let sending = &s.components.sending;
    assert_eq!(sending.status, ServiceState::Unknown);
    assert!(sending.by_stage.is_empty());
    assert_eq!(sending.longest_wait_seconds, None);
    assert_eq!(sending.sent_last_hour, 12);
    assert_eq!(sending.median_seconds, Some(2.0));
    assert_eq!(s.components.node.latency_ms, Some(250));
}

#[test]
fn service_status_never_measured() {
    let s: ServiceStatus = serde_json::from_str(STATUS_NEVER_MEASURED).unwrap();
    assert_eq!(s.status, ServiceState::Unknown);
    assert_eq!(s.updated_at, None);
    assert!(s.notice.is_none());
    let sending = &s.components.sending;
    assert!(sending.by_stage.is_empty());
    assert_eq!(sending.longest_wait_seconds, None);
    assert_eq!(sending.median_seconds, None);
    assert_eq!(sending.p95_seconds, None);
    assert_eq!(s.components.receiving.credit_backlog, None);
    assert_eq!(s.components.node.latency_ms, None);
    assert_eq!(s.components.webhooks.backlog, None);
}

#[test]
fn service_status_live_response() {
    let s: ServiceStatus = serde_json::from_str(STATUS_LIVE).unwrap();
    assert_eq!(s.status, ServiceState::Operational);
    // Seven fractional-second digits, kept as sent.
    assert_eq!(
        s.updated_at.as_deref(),
        Some("2026-10-25T17:16:11.9284738+00:00")
    );
    assert!(s.components.sending.by_stage.is_empty());
    assert_eq!(s.components.sending.median_seconds, None);
    assert_eq!(s.components.node.latency_ms, Some(885));
}

#[test]
fn service_status_unknown_values_and_fields() {
    // A state and a level this SDK doesn't know, a stage it doesn't know, and
    // fields added at every level, including a new component.
    let json = r#"{
        "status":"maintenance","updatedAt":"2026-10-25T14:33:53.06+00:00","region":"eu",
        "notice":{"level":"critical","message":"Planned work.","since":"2026-10-25T14:00:00+00:00","until":null,"link":"https://ln.bot/status"},
        "components":{
            "sending":{"status":"maintenance","inProgress":1,"byStage":{"teleporting":1},"longestWaitSeconds":0.5,"sentLastHour":0,"failedLastHour":0,"serviceFailuresLastHour":0,"medianSeconds":null,"p95Seconds":null,"extra":true},
            "receiving":{"status":"operational","creditedLastHour":0,"creditBacklog":0,"extra":1},
            "node":{"status":"operational","latencyMs":300,"extra":"x"},
            "webhooks":{"status":"operational","backlog":0,"extra":[]},
            "swaps":{"status":"operational"}
        }
    }"#;
    let s: ServiceStatus = serde_json::from_str(json).unwrap();
    assert_eq!(s.status, ServiceState::Other("maintenance".into()));
    assert_eq!(s.status.to_string(), "maintenance");
    let notice = s.notice.unwrap();
    assert_eq!(notice.level, NoticeLevel::Other("critical".into()));
    assert_eq!(notice.message, "Planned work.");
    assert_eq!(
        s.components.sending.status,
        ServiceState::Other("maintenance".into())
    );
    assert_eq!(s.components.sending.by_stage["teleporting"], 1);
    assert_eq!(s.components.sending.longest_wait_seconds, Some(0.5));
}

#[test]
fn status_history_90d() {
    let h: ServiceStatusHistory = serde_json::from_str(HISTORY_90D).unwrap();
    assert_eq!(h.range, "90d");
    assert_eq!(h.bucket_seconds, 86400);
    assert_eq!(h.from, "2026-07-27T00:00:00+00:00");
    assert_eq!(h.to, "2026-10-25T14:35:12+00:00");

    let sum = &h.summary;
    assert_eq!(sum.samples, 2850);
    assert_eq!(sum.uptime.overall.operational, Some(99.123));
    assert_eq!(sum.uptime.overall.degraded, Some(0.0));
    assert_eq!(sum.uptime.overall.outage, Some(0.877));
    assert_eq!(sum.uptime.webhooks.operational, Some(100.0));
    assert_eq!(sum.sent, 2410);
    assert_eq!(sum.failed, 31);
    assert_eq!(sum.service_failures, 6);
    assert_eq!(sum.credited, 5012);
    assert_eq!(sum.median_seconds, Some(2.2));
    assert_eq!(sum.p95_seconds, Some(7.9));
    assert_eq!(sum.avg_latency_ms, Some(301.0));
    assert_eq!(sum.max_latency_ms, Some(20000));
    let bounds: Vec<_> = sum.settle_times.iter().map(|t| t.up_to_seconds).collect();
    assert_eq!(
        bounds,
        [
            Some(1.0),
            Some(2.0),
            Some(3.0),
            Some(5.0),
            Some(10.0),
            Some(20.0),
            Some(30.0),
            Some(60.0),
            Some(120.0),
            Some(300.0),
            Some(600.0),
            None
        ]
    );
    assert_eq!(sum.settle_times[1].count, 1100);
    assert_eq!(sum.settle_times[11].count, 0);

    assert_eq!(h.buckets.len(), 3);
    let measured = &h.buckets[0];
    assert_eq!(measured.at, "2026-10-23T00:00:00+00:00");
    assert_eq!(measured.samples, 1425);
    assert_eq!(measured.sending.sent, 1200);
    assert_eq!(measured.sending.p95_seconds, Some(7.5));
    assert_eq!(measured.sending.max_in_progress, Some(4));
    assert_eq!(measured.sending.max_wait_seconds, Some(41.0));
    assert_eq!(measured.receiving.max_backlog, Some(3));
    assert_eq!(measured.node.avg_latency_ms, Some(290.0));
    assert_eq!(measured.webhooks.max_backlog, Some(4));

    let outage = &h.buckets[1];
    assert_eq!(outage.overall.outage, 25);
    assert_eq!(outage.overall.degraded, 0);
    assert_eq!(outage.sending.outage, 25);
    assert_eq!(outage.node.max_latency_ms, Some(20000));
    assert_eq!(outage.webhooks.outage, 0);

    let unmeasured = &h.buckets[2];
    assert_eq!(unmeasured.samples, 0);
    assert_eq!(unmeasured.overall, h.buckets[0].overall);
    assert_eq!(unmeasured.sending.sent, 0);
    assert_eq!(unmeasured.sending.median_seconds, None);
    assert_eq!(unmeasured.sending.p95_seconds, None);
    assert_eq!(unmeasured.sending.max_in_progress, Some(0));
    assert_eq!(unmeasured.sending.max_wait_seconds, None);
    assert_eq!(unmeasured.receiving.max_backlog, None);
    assert_eq!(unmeasured.node.avg_latency_ms, None);
    assert_eq!(unmeasured.node.max_latency_ms, None);
    assert_eq!(unmeasured.webhooks.max_backlog, None);

    assert_eq!(h.notices.len(), 2);
    assert_eq!(h.notices[0].level, NoticeLevel::Outage);
    assert_eq!(
        h.notices[0].until.as_deref(),
        Some("2026-10-24T09:32:00+00:00")
    );
    assert_eq!(h.notices[1].level, NoticeLevel::Info);
    assert_eq!(
        h.notices[1].message,
        "Payments are flowing again. We are watching it."
    );
}

#[test]
fn status_history_24h_live_response() {
    let h: ServiceStatusHistory = serde_json::from_str(HISTORY_24H_LIVE).unwrap();
    assert_eq!(h.range, "24h");
    assert_eq!(h.bucket_seconds, 300);
    assert_eq!(h.buckets.len(), 289);
    assert_eq!(h.to, "2026-10-25T17:16:44.1635789+00:00");
    assert_eq!(h.summary.settle_times.len(), 12);
    assert!(h.notices.is_empty());

    let first = &h.buckets[0];
    assert_eq!(first.at, "2026-10-24T17:15:00+00:00");
    assert_eq!(first.samples, 0);
    assert_eq!(first.sending.median_seconds, None);
    assert_eq!(first.sending.max_wait_seconds, None);
    assert_eq!(first.receiving.max_backlog, None);
    assert_eq!(first.node.avg_latency_ms, None);
    assert_eq!(first.node.max_latency_ms, None);
    assert_eq!(first.webhooks.max_backlog, None);

    let last = h.buckets.last().unwrap();
    assert_eq!(last.samples, 1);
    assert_eq!(last.node.avg_latency_ms, Some(885.0));
    assert_eq!(last.node.max_latency_ms, Some(885));
}

#[test]
fn status_history_nothing_measured() {
    // A range with no measurements, and a fractional average latency in a
    // bucket of another response.
    let json = r#"{
        "range":"7d","bucketSeconds":3600,"from":"2026-10-18T15:00:00+00:00","to":"2026-10-25T14:35:12+00:00",
        "summary":{"samples":0,
            "uptime":{"overall":{"operational":null,"degraded":null,"outage":null},"sending":{"operational":null,"degraded":null,"outage":null},"receiving":{"operational":null,"degraded":null,"outage":null},"node":{"operational":null,"degraded":null,"outage":null},"webhooks":{"operational":null,"degraded":null,"outage":null}},
            "sent":0,"failed":0,"serviceFailures":0,"credited":0,"medianSeconds":null,"p95Seconds":null,
            "settleTimes":[{"upToSeconds":1,"count":0},{"upToSeconds":null,"count":0}],
            "avgLatencyMs":null,"maxLatencyMs":null},
        "buckets":[{"at":"2026-10-18T15:00:00+00:00","samples":3,"overall":{"degraded":1,"outage":0},
            "sending":{"degraded":1,"outage":0,"sent":0,"failed":0,"serviceFailures":0,"medianSeconds":null,"p95Seconds":null,"maxInProgress":null,"maxWaitSeconds":null},
            "receiving":{"degraded":0,"outage":0,"credited":0,"maxBacklog":0},
            "node":{"degraded":0,"outage":0,"avgLatencyMs":296.4,"maxLatencyMs":310},
            "webhooks":{"degraded":0,"outage":0,"maxBacklog":0}}],
        "notices":[{"level":"warning","message":"Sends are slower than usual.","since":"2026-10-25T09:10:00+00:00","until":null}]
    }"#;
    let h: ServiceStatusHistory = serde_json::from_str(json).unwrap();
    assert_eq!(h.summary.samples, 0);
    assert_eq!(h.summary.uptime.overall.operational, None);
    assert_eq!(h.summary.uptime.node.outage, None);
    assert_eq!(h.summary.median_seconds, None);
    assert_eq!(h.summary.avg_latency_ms, None);
    assert_eq!(h.summary.max_latency_ms, None);
    assert_eq!(h.summary.settle_times[1].up_to_seconds, None);
    assert_eq!(h.buckets[0].sending.max_in_progress, None);
    assert_eq!(h.buckets[0].node.avg_latency_ms, Some(296.4));
    assert_eq!(h.notices[0].level, NoticeLevel::Warning);
    assert_eq!(h.notices[0].until, None);
}

// ---------------------------------------------------------------------------
// Event type conversions (From<&str>)
// ---------------------------------------------------------------------------

#[test]
fn invoice_event_type_from_str() {
    assert_eq!(InvoiceEventType::from("settled"), InvoiceEventType::Settled);
    assert_eq!(InvoiceEventType::from("expired"), InvoiceEventType::Expired);
    assert_eq!(InvoiceEventType::from("timeout"), InvoiceEventType::Timeout);
    assert_eq!(
        InvoiceEventType::from("new_event"),
        InvoiceEventType::Unknown("new_event".to_string()),
    );
}

#[test]
fn payment_event_type_from_str() {
    assert_eq!(PaymentEventType::from("settled"), PaymentEventType::Settled);
    assert_eq!(PaymentEventType::from("failed"), PaymentEventType::Failed);
    assert_eq!(PaymentEventType::from("timeout"), PaymentEventType::Timeout);
    assert_eq!(
        PaymentEventType::from("new_event"),
        PaymentEventType::Unknown("new_event".to_string()),
    );
}

// ---------------------------------------------------------------------------
// Error display
// ---------------------------------------------------------------------------

#[test]
fn error_display_bad_request() {
    let e = LnBotError::BadRequest {
        body: "invalid".into(),
    };
    assert_eq!(e.to_string(), "Bad Request (400): invalid");
}

#[test]
fn error_display_unauthorized() {
    let e = LnBotError::Unauthorized {
        body: "no key".into(),
    };
    assert_eq!(e.to_string(), "Unauthorized (401): no key");
}

#[test]
fn error_display_forbidden() {
    let e = LnBotError::Forbidden {
        body: "denied".into(),
    };
    assert_eq!(e.to_string(), "Forbidden (403): denied");
}

#[test]
fn error_display_not_found() {
    let e = LnBotError::NotFound {
        body: "missing".into(),
    };
    assert_eq!(e.to_string(), "Not Found (404): missing");
}

#[test]
fn error_display_conflict() {
    let e = LnBotError::Conflict {
        body: "exists".into(),
    };
    assert_eq!(e.to_string(), "Conflict (409): exists");
}

#[test]
fn error_display_api() {
    let e = LnBotError::Api {
        status: 503,
        body: "unavailable".into(),
    };
    assert_eq!(e.to_string(), "API error (HTTP 503): unavailable");
}
