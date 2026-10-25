use std::collections::HashMap;

use lnbot::*;

// ---------------------------------------------------------------------------
// Wallets (account-level)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn wallets_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"Bot","address":"a@ln.bot"}"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    let resp = client.wallets().create().await.unwrap();
    assert_eq!(resp.wallet_id, "wal_1");
    assert_eq!(resp.name, "Bot");
    assert_eq!(resp.address, "a@ln.bot");
    mock.assert_async().await;
}

#[tokio::test]
async fn wallets_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"walletId":"wal_1","name":"My Wallet"}]"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    let list = client.wallets().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].wallet_id, "wal_1");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Wallet Key
// ---------------------------------------------------------------------------

#[tokio::test]
async fn wallet_key_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"key":"wk_abc"}"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    let resp = client.wallet("wal_1").key().create().await.unwrap();
    assert_eq!(resp.key, "wk_abc");
    mock.assert_async().await;
}

#[tokio::test]
async fn wallet_key_get() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hint":"wk_abc...xyz","createdAt":"2024-01-01T00:00:00Z","lastUsedAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    let resp = client.wallet("wal_1").key().get().await.unwrap();
    assert_eq!(resp.hint, "wk_abc...xyz");
    mock.assert_async().await;
}

#[tokio::test]
async fn wallet_key_delete() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/wallets/wal_1/key")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    client.wallet("wal_1").key().delete().await.unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn wallet_key_rotate() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/key/rotate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"key":"wk_new"}"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_test").with_base_url(server.url());
    let resp = client.wallet("wal_1").key().rotate().await.unwrap();
    assert_eq!(resp.key, "wk_new");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Keys (account-level)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn keys_rotate() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/keys/1/rotate")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"key":"key_new","name":"primary"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client.keys().rotate(1).await.unwrap();
    assert_eq!(resp.key, "key_new");
    assert_eq!(resp.name, "primary");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Invoices (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invoices_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/invoices")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"amount": 100, "memo": "test"}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"pending","amount":100,"bolt11":"lnbc1...","reference":null,"memo":"test","preimage":null,"txNumber":null,"createdAt":null,"settledAt":null,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let inv = client
        .wallet("wal_1")
        .invoices()
        .create(&CreateInvoiceRequest::new(100).memo("test"))
        .await
        .unwrap();
    assert_eq!(inv.number, 1);
    assert_eq!(inv.status, InvoiceStatus::Pending);
    assert_eq!(inv.amount, 100);
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "5".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"status":"settled","amount":100,"bolt11":"lnbc1...","reference":null,"memo":null,"preimage":null,"txNumber":null,"createdAt":null,"settledAt":null,"expiresAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .wallet("wal_1")
        .invoices()
        .list(&ListParams::default().limit(5))
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].number, 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_list_with_pagination() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "10".into()),
            mockito::Matcher::UrlEncoded("after".into(), "5".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("[]")
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .wallet("wal_1")
        .invoices()
        .list(&ListParams::default().limit(10).after(5))
        .await
        .unwrap();
    assert!(list.is_empty());
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_get() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":42,"status":"settled","amount":500,"bolt11":"lnbc5...","reference":null,"memo":null,"preimage":"abc","txNumber":10,"createdAt":null,"settledAt":null,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let inv = client.wallet("wal_1").invoices().get(42).await.unwrap();
    assert_eq!(inv.number, 42);
    assert_eq!(inv.status, InvoiceStatus::Settled);
    assert_eq!(inv.preimage.as_deref(), Some("abc"));
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_get_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/invoices/abc123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"pending","amount":100,"bolt11":"lnbc1...","reference":null,"memo":null,"preimage":null,"txNumber":null,"createdAt":null,"settledAt":null,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let inv = client
        .wallet("wal_1")
        .invoices()
        .get_by_hash("abc123")
        .await
        .unwrap();
    assert_eq!(inv.number, 1);
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Public Invoices (top-level, no auth)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invoices_create_for_wallet() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/invoices/for-wallet")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"walletId": "wid", "amount": 200}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"bolt11":"lnbc2...","amount":200,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let resp = client
        .invoices()
        .create_for_wallet(&CreateInvoiceForWalletRequest::new("wid", 200))
        .await
        .unwrap();
    assert_eq!(resp.bolt11, "lnbc2...");
    assert_eq!(resp.amount, 200);
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_create_for_address() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/invoices/for-address")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"address": "user@ln.bot", "amount": 300}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"bolt11":"lnbc3...","amount":300,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let resp = client
        .invoices()
        .create_for_address(&CreateInvoiceForAddressRequest::new("user@ln.bot", 300))
        .await
        .unwrap();
    assert_eq!(resp.bolt11, "lnbc3...");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Payments (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn payments_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/payments")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"target": "user@ln.bot", "amount": 50}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"pending","amount":50,"maxFee":10,"serviceFee":0,"actualFee":null,"address":"user@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let p = client
        .wallet("wal_1")
        .payments()
        .create(&CreatePaymentRequest::new("user@ln.bot").amount(50))
        .await
        .unwrap();
    assert_eq!(p.number, 1);
    assert_eq!(p.status, PaymentStatus::Pending);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":2,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .wallet("wal_1")
        .payments()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_get() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":7,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":1,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let p = client.wallet("wal_1").payments().get(7).await.unwrap();
    assert_eq!(p.number, 7);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_list_with_and_without_stage() {
    // A payment being retried, one from an API without the stage fields, and
    // one with a stage this SDK doesn't know.
    let body = r#"[
        {"number":4,"status":"pending","amount":1000,"maxFee":20,"serviceFee":3,"actualFee":null,"address":"lnbc10u1p...","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":"2026-10-25T14:33:53+00:00","settledAt":null,"stage":"retrying","statusDetail":"Not sent yet; retrying for a few minutes. If it still can't be sent, it fails and the funds are released.","retryUntil":"2026-10-25T14:43:53+00:00"},
        {"number":3,"status":"settled","amount":500,"maxFee":20,"serviceFee":1,"actualFee":2,"address":"x4k7m2@ln.bot","reference":null,"preimage":"9f2c...","txNumber":12,"failureReason":null,"createdAt":"2026-10-25T14:30:00+00:00","settledAt":"2026-10-25T14:30:01+00:00"},
        {"number":5,"status":"processing","amount":1000,"maxFee":20,"serviceFee":3,"actualFee":null,"address":"x4k7m2@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":"2026-10-25T14:35:00+00:00","settledAt":null,"stage":"teleporting","statusDetail":"Moving through a new kind of route.","retryUntil":null}
    ]"#;
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .wallet("wal_1")
        .payments()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].stage, Some(PaymentStage::Retrying));
    assert_eq!(
        list[0].retry_until.as_deref(),
        Some("2026-10-25T14:43:53+00:00")
    );
    assert!(list[0].status_detail.is_some());
    assert_eq!(list[1].stage, None);
    assert_eq!(list[1].status_detail, None);
    assert_eq!(list[1].retry_until, None);
    assert_eq!(list[2].stage, Some(PaymentStage::Unknown));
    assert_eq!(list[2].status, PaymentStatus::Processing);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_get_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/hash123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":null,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client
        .wallet("wal_1")
        .payments()
        .get_by_hash("hash123")
        .await
        .unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_resolve() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/resolve")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("target".into(), "user@ln.bot".into()),
            // Only the target is sent: no amount parameter.
            mockito::Matcher::Regex("^target=[^&]*$".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"type":"lightning_address","min":1,"max":1000000000,"fixed":false,"amount":null,"internal":false,"networkFeeEstimate":null,"serviceFee":null,"maxFee":null,"total":null,"maxSendable":49750,"feeWarning":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .payments()
        .resolve("user@ln.bot")
        .await
        .unwrap();
    assert_eq!(resp.target_type, "lightning_address");
    assert!(resp.amount.is_none());
    assert!(resp.total.is_none());
    assert_eq!(resp.max_sendable, Some(49750));
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_resolve_with_amount() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/payments/resolve")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("target".into(), "user@ln.bot".into()),
            mockito::Matcher::UrlEncoded("amount".into(), "10000".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"type":"lightning_address","min":1,"max":1000000000,"fixed":false,"amount":10000,"internal":false,"networkFeeEstimate":27,"serviceFee":25,"maxFee":31,"total":10052,"maxSendable":null,"feeWarning":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .payments()
        .resolve_with_amount("user@ln.bot", 10000)
        .await
        .unwrap();
    assert_eq!(resp.amount, Some(10000));
    assert_eq!(resp.network_fee_estimate, Some(27));
    assert_eq!(resp.service_fee, Some(25));
    assert_eq!(resp.max_fee, Some(31));
    assert_eq!(resp.total, Some(10052));
    assert!(resp.max_sendable.is_none());
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Addresses (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn addresses_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/addresses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"address":"bot@ln.bot","generated":false,"cost":0,"createdAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let addr = client
        .wallet("wal_1")
        .addresses()
        .create(&CreateAddressRequest {
            address: Some("bot@ln.bot".into()),
        })
        .await
        .unwrap();
    assert_eq!(addr.address, "bot@ln.bot");
    assert!(!addr.generated);
    mock.assert_async().await;
}

#[tokio::test]
async fn addresses_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/addresses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"address":"a@ln.bot","generated":true,"cost":0,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.wallet("wal_1").addresses().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].generated);
    mock.assert_async().await;
}

#[tokio::test]
async fn addresses_delete_url_encodes() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/wallets/wal_1/addresses/user%40ln.bot")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client
        .wallet("wal_1")
        .addresses()
        .delete("user@ln.bot")
        .await
        .unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn addresses_transfer() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/addresses/user%40ln.bot/transfer")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"targetWalletKey": "key_other"}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"address":"user@ln.bot","transferredTo":"w2"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .addresses()
        .transfer("user@ln.bot", &TransferAddressRequest::new("key_other"))
        .await
        .unwrap();
    assert_eq!(resp.address, "user@ln.bot");
    assert_eq!(resp.transferred_to, "w2");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Transactions (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn transactions_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/transactions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"type":"credit","amount":100,"balanceAfter":100,"networkFee":0,"serviceFee":0,"paymentHash":null,"preimage":null,"reference":null,"note":null,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .wallet("wal_1")
        .transactions()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].tx_type, TransactionType::Credit);
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Webhooks (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhooks_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/webhooks")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"url": "https://example.com/hook"}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"wh1","url":"https://example.com/hook","secret":"sec123","createdAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let wh = client
        .wallet("wal_1")
        .webhooks()
        .create(&CreateWebhookRequest::new("https://example.com/hook"))
        .await
        .unwrap();
    assert_eq!(wh.id, "wh1");
    assert_eq!(wh.secret, "sec123");
    mock.assert_async().await;
}

#[tokio::test]
async fn webhooks_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"wh1","url":"https://example.com","active":true,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.wallet("wal_1").webhooks().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].active);
    mock.assert_async().await;
}

#[tokio::test]
async fn webhooks_delete() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/wallets/wal_1/webhooks/wh-123")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client
        .wallet("wal_1")
        .webhooks()
        .delete("wh-123")
        .await
        .unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Backup (account-level)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn backup_recovery() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/backup/recovery")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"passphrase":"word1 word2 word3"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client.backup().recovery().await.unwrap();
    assert_eq!(resp.passphrase, "word1 word2 word3");
    mock.assert_async().await;
}

#[tokio::test]
async fn backup_passkey_begin() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/backup/passkey/begin")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"sessionId":"sess1","options":{"rp":{"name":"LnBot"}}}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client.backup().passkey_begin().await.unwrap();
    assert_eq!(resp.session_id, "sess1");
    assert!(resp.options.contains_key("rp"));
    mock.assert_async().await;
}

#[tokio::test]
async fn backup_passkey_complete() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/backup/passkey/complete")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client
        .backup()
        .passkey_complete(&BackupPasskeyCompleteRequest {
            session_id: "sess1".into(),
            attestation: HashMap::new(),
        })
        .await
        .unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Restore (account-level)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn restore_recovery() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/restore/recovery")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"passphrase": "word1 word2 word3"}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"w1","name":"Restored","primaryKey":"pk","secondaryKey":"sk"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .restore()
        .recovery(&RecoveryRestoreRequest::new("word1 word2 word3"))
        .await
        .unwrap();
    assert_eq!(resp.wallet_id, "w1");
    assert_eq!(resp.primary_key, "pk");
    mock.assert_async().await;
}

#[tokio::test]
async fn restore_passkey_begin() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/restore/passkey/begin")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"sessionId":"sess2","options":{}}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client.restore().passkey_begin().await.unwrap();
    assert_eq!(resp.session_id, "sess2");
    mock.assert_async().await;
}

#[tokio::test]
async fn restore_passkey_complete() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/restore/passkey/complete")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"w1","name":"Restored","primaryKey":"pk","secondaryKey":"sk"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .restore()
        .passkey_complete(&RestorePasskeyCompleteRequest {
            session_id: "sess2".into(),
            assertion: HashMap::new(),
        })
        .await
        .unwrap();
    assert_eq!(resp.wallet_id, "w1");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// L402 (wallet-scoped)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn l402_create_challenge() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/l402/challenges")
        .match_body(mockito::Matcher::Json(serde_json::json!({"amount": 100})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"macaroon":"mac","invoice":"lnbc1...","paymentHash":"hash","expiresAt":"2024-01-01T00:00:00Z","wwwAuthenticate":"L402 mac:inv"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .l402()
        .create_challenge(&CreateL402ChallengeRequest {
            amount: 100,
            description: None,
            expiry_seconds: None,
            caveats: None,
        })
        .await
        .unwrap();
    assert_eq!(resp.macaroon, "mac");
    assert_eq!(resp.payment_hash, "hash");
    mock.assert_async().await;
}

#[tokio::test]
async fn l402_verify() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/l402/verify")
        .match_body(mockito::Matcher::Json(
            serde_json::json!({"authorization": "L402 token"}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"valid":true,"paymentHash":"hash","caveats":null,"error":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .l402()
        .verify(&VerifyL402Request {
            authorization: "L402 token".into(),
        })
        .await
        .unwrap();
    assert!(resp.valid);
    mock.assert_async().await;
}

#[tokio::test]
async fn l402_pay() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets/wal_1/l402/pay")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization":"L402 final","paymentHash":"hash","preimage":"pre","amount":100,"fee":1,"paymentNumber":1,"status":"settled"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
        .wallet("wal_1")
        .l402()
        .pay(&PayL402Request {
            www_authenticate: "L402 mac:inv".into(),
            max_fee: None,
            reference: None,
            wait: Some(true),
            timeout: None,
        })
        .await
        .unwrap();
    assert_eq!(resp.authorization.as_deref(), Some("L402 final"));
    assert_eq!(resp.status, "settled");
    mock.assert_async().await;
}
