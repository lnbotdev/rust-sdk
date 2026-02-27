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
fn create_wallet_request_skips_none_name() {
    let req = CreateWalletRequest::default();
    let json = serde_json::to_value(&req).unwrap();
    assert!(json.get("name").is_none());
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
fn transaction_type_unknown_falls_back() {
    assert_eq!(
        serde_json::from_str::<TransactionType>(r#""future_type""#).unwrap(),
        TransactionType::Unknown,
    );
}

// ---------------------------------------------------------------------------
// Event type conversions (From<&str>)
// ---------------------------------------------------------------------------

#[test]
fn invoice_event_type_from_str() {
    assert_eq!(InvoiceEventType::from("settled"), InvoiceEventType::Settled);
    assert_eq!(InvoiceEventType::from("expired"), InvoiceEventType::Expired);
    assert_eq!(
        InvoiceEventType::from("new_event"),
        InvoiceEventType::Unknown("new_event".to_string()),
    );
}

#[test]
fn payment_event_type_from_str() {
    assert_eq!(PaymentEventType::from("settled"), PaymentEventType::Settled);
    assert_eq!(PaymentEventType::from("failed"), PaymentEventType::Failed);
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
    let e = LnBotError::BadRequest { body: "invalid".into() };
    assert_eq!(e.to_string(), "Bad Request (400): invalid");
}

#[test]
fn error_display_unauthorized() {
    let e = LnBotError::Unauthorized { body: "no key".into() };
    assert_eq!(e.to_string(), "Unauthorized (401): no key");
}

#[test]
fn error_display_forbidden() {
    let e = LnBotError::Forbidden { body: "denied".into() };
    assert_eq!(e.to_string(), "Forbidden (403): denied");
}

#[test]
fn error_display_not_found() {
    let e = LnBotError::NotFound { body: "missing".into() };
    assert_eq!(e.to_string(), "Not Found (404): missing");
}

#[test]
fn error_display_conflict() {
    let e = LnBotError::Conflict { body: "exists".into() };
    assert_eq!(e.to_string(), "Conflict (409): exists");
}

#[test]
fn error_display_api() {
    let e = LnBotError::Api { status: 503, body: "unavailable".into() };
    assert_eq!(e.to_string(), "API error (HTTP 503): unavailable");
}
