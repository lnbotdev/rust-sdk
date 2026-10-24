use futures_util::StreamExt;
use lnbot::*;

// Payloads exactly as the API sends them: event data uses snake_case keys.

const PAYMENT_SETTLED: &str = r#"{"number":1,"status":"settled","amount":20,"max_fee":5,"service_fee":1,"actual_fee":3,"address":"bub@bub.gg","reference":null,"preimage":"ab12...","tx_number":4,"failure_reason":null,"created_at":"2026-10-24T14:50:01Z","settled_at":"2026-10-24T14:50:14Z"}"#;

const PAYMENT_FAILED: &str = r#"{"number":3,"status":"failed","amount":20,"max_fee":20,"service_fee":1,"actual_fee":null,"address":"bub@bub.gg","reference":null,"preimage":null,"tx_number":null,"failure_reason":"Invoice expired before it could be paid.","created_at":"2026-10-24T14:50:01Z","settled_at":null}"#;

const PAYMENT_PROCESSING: &str = r#"{"number":1,"status":"processing","amount":20,"max_fee":5,"service_fee":1,"actual_fee":null,"address":"bub@bub.gg","reference":null,"preimage":null,"tx_number":null,"failure_reason":null,"created_at":"2026-10-24T14:50:01Z","settled_at":null}"#;

// `payment.created` on the wallet stream leaves out the fields set later.
const PAYMENT_CREATED: &str = r#"{"number":2,"status":"pending","amount":20,"max_fee":5,"service_fee":1,"address":"bub@bub.gg","reference":null,"created_at":"2026-10-24T14:50:01Z"}"#;

const INVOICE_SETTLED: &str = r#"{"number":7,"status":"settled","amount":1000,"bolt11":"lnbc10u1p...","reference":null,"memo":null,"preimage":"cd34...","tx_number":4,"created_at":"2026-10-24T14:49:00Z","settled_at":"2026-10-24T14:50:14Z","expires_at":"2026-10-25T14:49:00Z"}"#;

const INVOICE_PENDING: &str = r#"{"number":7,"status":"pending","amount":1000,"bolt11":"lnbc10u1p...","reference":null,"memo":null,"preimage":null,"tx_number":null,"created_at":"2026-10-24T14:49:00Z","settled_at":null,"expires_at":"2026-10-25T14:49:00Z"}"#;

const INVOICE_CREATED: &str = r#"{"number":8,"status":"pending","amount":1000,"bolt11":"lnbc10u1p...","reference":null,"memo":null,"created_at":"2026-10-24T14:49:00Z","expires_at":"2026-10-25T14:49:00Z"}"#;

fn wallet_event(event: &str, data: &str) -> String {
    format!(
        "data: {{\"event\":\"{event}\",\"created_at\":\"2026-10-24T14:50:14.123+00:00\",\"data\":{data}}}\n\n"
    )
}

// ---------------------------------------------------------------------------
// Invoice watch (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invoice_watch_yields_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("event: settled\ndata: {\"number\":1,\"status\":\"settled\",\"amount\":100,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":\"abc\",\"tx_number\":null,\"created_at\":null,\"settled_at\":null,\"expires_at\":null}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, InvoiceEventType::Settled);
    assert_eq!(events[0].data.number, 1);
    assert_eq!(events[0].data.amount, 100);
}

#[tokio::test]
async fn invoice_watch_multiple_events() {
    let body = "\
        event: pending\n\
        data: {\"number\":1,\"status\":\"pending\",\"amount\":50,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":null,\"tx_number\":null,\"created_at\":null,\"settled_at\":null,\"expires_at\":null}\n\
        \n\
        event: settled\n\
        data: {\"number\":1,\"status\":\"settled\",\"amount\":50,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":\"abc\",\"tx_number\":null,\"created_at\":null,\"settled_at\":null,\"expires_at\":null}\n\
        \n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, InvoiceEventType::from("pending"));
    assert_eq!(events[1].event, InvoiceEventType::Settled);
}

#[tokio::test]
async fn invoice_watch_skips_comments() {
    let body = "\
        : keepalive\n\
        \n\
        event: settled\n\
        data: {\"number\":1,\"status\":\"settled\",\"amount\":100,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":null,\"tx_number\":null,\"created_at\":null,\"settled_at\":null,\"expires_at\":null}\n\
        \n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
}

#[tokio::test]
async fn invoice_watch_empty_stream() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(events.is_empty());
}

#[tokio::test]
async fn invoice_watch_with_timeout() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/42/events?timeout=120")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(42, Some(120))
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_sends_sse_accept_header() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .match_header("accept", "text/event-stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_sends_authorization() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .match_header("authorization", "Bearer key_test")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/abc123/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch_by_hash("abc123", None)
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/1/events")
        .with_status(401)
        .with_body(r#"{"message":"unauthorized"}"#)
        .create_async()
        .await;

    let client = LnBot::new("bad_key").with_base_url(server.url());
    let results: Vec<_> = client
        .wallet("wal_1")
        .invoices()
        .watch(1, None)
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Unauthorized { .. }));
}

#[tokio::test]
async fn invoice_watch_decodes_snake_case_wire_format() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/7/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!("event: settled\ndata: {INVOICE_SETTLED}\n\n"))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
        .wallet("wal_1")
        .invoices()
        .watch(7, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, InvoiceEventType::Settled);
    let inv = &events[0].data;
    assert_eq!(inv.number, 7);
    assert_eq!(inv.status, InvoiceStatus::Settled);
    assert_eq!(inv.amount, 1000);
    assert_eq!(inv.bolt11, "lnbc10u1p...");
    assert_eq!(inv.preimage.as_deref(), Some("cd34..."));
    assert_eq!(inv.tx_number, Some(4));
    assert_eq!(inv.created_at.as_deref(), Some("2026-10-24T14:49:00Z"));
    assert_eq!(inv.settled_at.as_deref(), Some("2026-10-24T14:50:14Z"));
    assert_eq!(inv.expires_at.as_deref(), Some("2026-10-25T14:49:00Z"));
}

#[tokio::test]
async fn invoice_watch_timeout_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/invoices/7/events?timeout=5")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!(
            ":keepalive\n\nevent: timeout\ndata: {INVOICE_PENDING}\n\n"
        ))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
        .wallet("wal_1")
        .invoices()
        .watch(7, Some(5))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, InvoiceEventType::Timeout);
    assert_eq!(events[0].data.status, InvoiceStatus::Pending);
    assert_eq!(events[0].data.settled_at, None);
    assert_eq!(
        events[0].data.expires_at.as_deref(),
        Some("2026-10-25T14:49:00Z")
    );
}

// ---------------------------------------------------------------------------
// Payment watch (wallet-scoped)
// ---------------------------------------------------------------------------

// The API sends snake_case. camelCase keys must keep working too, in case it
// ever switches to the REST spelling.
#[tokio::test]
async fn payment_watch_camel_case_still_decodes() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("event: settled\ndata: {\"number\":1,\"status\":\"settled\",\"amount\":50,\"maxFee\":10,\"serviceFee\":0,\"actualFee\":1,\"address\":\"user@ln.bot\",\"reference\":null,\"preimage\":null,\"txNumber\":null,\"failureReason\":null,\"createdAt\":null,\"settledAt\":null}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<PaymentEvent> = client
        .wallet("wal_1")
        .payments()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, PaymentEventType::Settled);
    assert_eq!(events[0].data.amount, 50);
    assert_eq!(events[0].data.max_fee, 10);
    assert_eq!(events[0].data.actual_fee, Some(1));
}

#[tokio::test]
async fn payment_watch_with_timeout() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/7/events?timeout=60")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .payments()
        .watch(7, Some(60))
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn payment_watch_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/hash123/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client
        .wallet("wal_1")
        .payments()
        .watch_by_hash("hash123", None)
        .collect()
        .await;
    mock.assert_async().await;
}

#[tokio::test]
async fn payment_watch_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/1/events")
        .with_status(403)
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client
        .wallet("wal_1")
        .payments()
        .watch(1, None)
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Forbidden { .. }));
}

#[tokio::test]
async fn payment_watch_decodes_snake_case_wire_format() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!("event: settled\ndata: {PAYMENT_SETTLED}\n\n"))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<PaymentEvent> = client
        .wallet("wal_1")
        .payments()
        .watch(1, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, PaymentEventType::Settled);
    let p = &events[0].data;
    assert_eq!(p.number, 1);
    assert_eq!(p.status, PaymentStatus::Settled);
    assert_eq!(p.amount, 20);
    assert_eq!(p.max_fee, 5);
    assert_eq!(p.service_fee, 1);
    assert_eq!(p.actual_fee, Some(3));
    assert_eq!(p.address, "bub@bub.gg");
    assert_eq!(p.preimage.as_deref(), Some("ab12..."));
    assert_eq!(p.tx_number, Some(4));
    assert_eq!(p.failure_reason, None);
    assert_eq!(p.created_at.as_deref(), Some("2026-10-24T14:50:01Z"));
    assert_eq!(p.settled_at.as_deref(), Some("2026-10-24T14:50:14Z"));
}

#[tokio::test]
async fn payment_watch_failed_event_has_failure_reason() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/3/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!("event: failed\ndata: {PAYMENT_FAILED}\n\n"))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<PaymentEvent> = client
        .wallet("wal_1")
        .payments()
        .watch(3, None)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, PaymentEventType::Failed);
    let p = &events[0].data;
    assert_eq!(p.status, PaymentStatus::Failed);
    assert_eq!(p.max_fee, 20);
    assert_eq!(p.service_fee, 1);
    assert_eq!(p.actual_fee, None);
    assert_eq!(
        p.failure_reason.as_deref(),
        Some("Invoice expired before it could be paid.")
    );
}

#[tokio::test]
async fn payment_watch_timeout_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/1/events?timeout=5")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!(
            ":keepalive\n\nevent: timeout\ndata: {PAYMENT_PROCESSING}\n\n"
        ))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<PaymentEvent> = client
        .wallet("wal_1")
        .payments()
        .watch(1, Some(5))
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, PaymentEventType::Timeout);
    assert_eq!(events[0].data.status, PaymentStatus::Processing);
    assert_eq!(events[0].data.max_fee, 5);
    assert_eq!(events[0].data.actual_fee, None);
}

#[tokio::test]
async fn payment_watch_undecodable_event_yields_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/payments/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("event: settled\ndata: {\"number\":1,\"status\":\"settled\"}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client
        .wallet("wal_1")
        .payments()
        .watch(1, None)
        .collect()
        .await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Json(_)));
}

// ---------------------------------------------------------------------------
// Events stream (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn events_stream_yields_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: {\"event\":\"invoice.settled\",\"created_at\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
        .wallet("wal_1")
        .events()
        .stream()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, "invoice.settled");
    assert_eq!(events[0].data["number"], 1);
}

#[tokio::test]
async fn events_stream_multiple() {
    let body = "\
        data: {\"event\":\"invoice.settled\",\"created_at\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n\
        data: {\"event\":\"payment.settled\",\"created_at\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":2}}\n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
        .wallet("wal_1")
        .events()
        .stream()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event, "invoice.settled");
    assert_eq!(events[1].event, "payment.settled");
}

#[tokio::test]
async fn events_stream_skips_non_data_lines() {
    let body = "\
        : keepalive\n\
        event: ignored\n\
        data: {\"event\":\"payment.settled\",\"created_at\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
        .wallet("wal_1")
        .events()
        .stream()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event, "payment.settled");
}

#[tokio::test]
async fn events_stream_sends_sse_headers() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/events")
        .match_header("accept", "text/event-stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.wallet("wal_1").events().stream().collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn events_stream_empty() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
        .wallet("wal_1")
        .events()
        .stream()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(events.is_empty());
}

#[tokio::test]
async fn events_stream_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(403)
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client.wallet("wal_1").events().stream().collect().await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Forbidden { .. }));
}

#[tokio::test]
async fn events_stream_decodes_snake_case_wire_format() {
    let body = [
        wallet_event("invoice.created", INVOICE_CREATED),
        wallet_event("invoice.settled", INVOICE_SETTLED),
        wallet_event("payment.created", PAYMENT_CREATED),
        wallet_event("payment.settled", PAYMENT_SETTLED),
        wallet_event("payment.failed", PAYMENT_FAILED),
    ]
    .concat();

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(format!(":keepalive\n\n{body}"))
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
        .wallet("wal_1")
        .events()
        .stream()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let names: Vec<_> = events.iter().map(|e| e.event.as_str()).collect();
    assert_eq!(
        names,
        [
            "invoice.created",
            "invoice.settled",
            "payment.created",
            "payment.settled",
            "payment.failed",
        ]
    );
    for e in &events {
        assert_eq!(e.created_at, "2026-10-24T14:50:14.123+00:00");
    }

    // `data` carries the REST (camelCase) keys.
    assert_eq!(events[3].data["maxFee"], 5);
    assert!(events[3].data.get("max_fee").is_none());

    let created: InvoiceResponse = serde_json::from_value(events[0].data.clone()).unwrap();
    assert_eq!(created.number, 8);
    assert_eq!(created.status, InvoiceStatus::Pending);
    assert_eq!(created.preimage, None);
    assert_eq!(created.tx_number, None);
    assert_eq!(created.settled_at, None);
    assert_eq!(created.created_at.as_deref(), Some("2026-10-24T14:49:00Z"));
    assert_eq!(created.expires_at.as_deref(), Some("2026-10-25T14:49:00Z"));

    let settled: InvoiceResponse = serde_json::from_value(events[1].data.clone()).unwrap();
    assert_eq!(settled.tx_number, Some(4));
    assert_eq!(settled.settled_at.as_deref(), Some("2026-10-24T14:50:14Z"));
    assert_eq!(settled.expires_at.as_deref(), Some("2026-10-25T14:49:00Z"));

    let created: PaymentResponse = serde_json::from_value(events[2].data.clone()).unwrap();
    assert_eq!(created.number, 2);
    assert_eq!(created.status, PaymentStatus::Pending);
    assert_eq!(created.max_fee, 5);
    assert_eq!(created.service_fee, 1);
    assert_eq!(created.actual_fee, None);
    assert_eq!(created.preimage, None);
    assert_eq!(created.tx_number, None);
    assert_eq!(created.failure_reason, None);
    assert_eq!(created.settled_at, None);
    assert_eq!(created.created_at.as_deref(), Some("2026-10-24T14:50:01Z"));

    let settled: PaymentResponse = serde_json::from_value(events[3].data.clone()).unwrap();
    assert_eq!(settled.max_fee, 5);
    assert_eq!(settled.service_fee, 1);
    assert_eq!(settled.actual_fee, Some(3));
    assert_eq!(settled.tx_number, Some(4));
    assert_eq!(settled.settled_at.as_deref(), Some("2026-10-24T14:50:14Z"));

    let failed: PaymentResponse = serde_json::from_value(events[4].data.clone()).unwrap();
    assert_eq!(failed.status, PaymentStatus::Failed);
    assert_eq!(
        failed.failure_reason.as_deref(),
        Some("Invoice expired before it could be paid.")
    );
}

#[tokio::test]
async fn events_stream_undecodable_event_yields_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: {\"event\":\"payment.settled\",\"data\":{}}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client.wallet("wal_1").events().stream().collect().await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Json(_)));
}
