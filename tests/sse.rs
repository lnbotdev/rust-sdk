use futures_util::StreamExt;
use lnbot::*;

// ---------------------------------------------------------------------------
// Invoice watch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invoice_watch_yields_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("event: settled\ndata: {\"number\":1,\"status\":\"settled\",\"amount\":100,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":\"abc\",\"txNumber\":null,\"createdAt\":null,\"settledAt\":null,\"expiresAt\":null}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
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
        data: {\"number\":1,\"status\":\"pending\",\"amount\":50,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":null,\"txNumber\":null,\"createdAt\":null,\"settledAt\":null,\"expiresAt\":null}\n\
        \n\
        event: settled\n\
        data: {\"number\":1,\"status\":\"settled\",\"amount\":50,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":\"abc\",\"txNumber\":null,\"createdAt\":null,\"settledAt\":null,\"expiresAt\":null}\n\
        \n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
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
        data: {\"number\":1,\"status\":\"settled\",\"amount\":100,\"bolt11\":\"lnbc1...\",\"reference\":null,\"memo\":null,\"preimage\":null,\"txNumber\":null,\"createdAt\":null,\"settledAt\":null,\"expiresAt\":null}\n\
        \n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<InvoiceEvent> = client
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
        .mock("GET", "/v1/invoices/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
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
        .mock("GET", "/v1/invoices/42/events?timeout=120")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.invoices().watch(42, Some(120)).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_sends_sse_accept_header() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/invoices/1/events")
        .match_header("accept", "text/event-stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.invoices().watch(1, None).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_sends_authorization() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/invoices/1/events")
        .match_header("authorization", "Bearer key_test")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.invoices().watch(1, None).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/invoices/abc123/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.invoices().watch_by_hash("abc123", None).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn invoice_watch_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/invoices/1/events")
        .with_status(401)
        .with_body(r#"{"message":"unauthorized"}"#)
        .create_async()
        .await;

    let client = LnBot::new("bad_key").with_base_url(server.url());
    let results: Vec<_> = client.invoices().watch(1, None).collect().await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Unauthorized { .. }));
}

// ---------------------------------------------------------------------------
// Payment watch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn payment_watch_yields_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/payments/1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("event: settled\ndata: {\"number\":1,\"status\":\"settled\",\"amount\":50,\"maxFee\":10,\"serviceFee\":0,\"actualFee\":1,\"address\":\"user@ln.bot\",\"reference\":null,\"preimage\":null,\"txNumber\":null,\"failureReason\":null,\"createdAt\":null,\"settledAt\":null}\n\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<PaymentEvent> = client
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
}

#[tokio::test]
async fn payment_watch_with_timeout() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/payments/7/events?timeout=60")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.payments().watch(7, Some(60)).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn payment_watch_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/payments/hash123/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.payments().watch_by_hash("hash123", None).collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn payment_watch_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/payments/1/events")
        .with_status(403)
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client.payments().watch(1, None).collect().await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Forbidden { .. }));
}

// ---------------------------------------------------------------------------
// Events stream
// ---------------------------------------------------------------------------

#[tokio::test]
async fn events_stream_yields_event() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("data: {\"event\":\"invoice.settled\",\"createdAt\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
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
        data: {\"event\":\"invoice.settled\",\"createdAt\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n\
        data: {\"event\":\"payment.settled\",\"createdAt\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":2}}\n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
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
        data: {\"event\":\"payment.settled\",\"createdAt\":\"2024-01-01T00:00:00Z\",\"data\":{\"number\":1}}\n";

    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<WalletEvent> = client
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
        .mock("GET", "/v1/events")
        .match_header("accept", "text/event-stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let _: Vec<_> = client.events().stream().collect().await;
    mock.assert_async().await;
}

#[tokio::test]
async fn events_stream_empty() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/events")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body("")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let events: Vec<_> = client
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
        .mock("GET", "/v1/events")
        .with_status(403)
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let results: Vec<_> = client.events().stream().collect().await;
    assert_eq!(results.len(), 1);
    let err = results[0].as_ref().unwrap_err();
    assert!(matches!(err, LnBotError::Forbidden { .. }));
}
