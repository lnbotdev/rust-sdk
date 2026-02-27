use std::collections::HashMap;

use lnbot::*;

// ---------------------------------------------------------------------------
// Wallets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn wallets_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/wallets")
        .match_body(mockito::Matcher::Json(serde_json::json!({"name": "Bot"})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"w1","primaryKey":"pk","secondaryKey":"sk","name":"Bot","address":"a@ln.bot","recoveryPassphrase":"w1 w2"}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let resp = client
        .wallets()
        .create(&CreateWalletRequest { name: Some("Bot".into()) })
        .await
        .unwrap();
    assert_eq!(resp.wallet_id, "w1");
    assert_eq!(resp.primary_key, "pk");
    assert_eq!(resp.address, "a@ln.bot");
    mock.assert_async().await;
}

#[tokio::test]
async fn wallets_current() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/current")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"w1","name":"My Wallet","balance":1000,"onHold":100,"available":900}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let w = client.wallets().current().await.unwrap();
    assert_eq!(w.wallet_id, "w1");
    assert_eq!(w.balance, 1000);
    assert_eq!(w.available, 900);
    mock.assert_async().await;
}

#[tokio::test]
async fn wallets_update() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PATCH", "/v1/wallets/current")
        .match_body(mockito::Matcher::Json(serde_json::json!({"name": "Renamed"})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"w1","name":"Renamed","balance":0,"onHold":0,"available":0}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let w = client.wallets().update(&UpdateWalletRequest::new("Renamed")).await.unwrap();
    assert_eq!(w.name, "Renamed");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Keys
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
// Invoices
// ---------------------------------------------------------------------------

#[tokio::test]
async fn invoices_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/invoices")
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
        .mock("GET", "/v1/invoices")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("limit".into(), "5".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"status":"settled","amount":100,"bolt11":"lnbc1...","reference":null,"memo":null,"preimage":null,"txNumber":null,"createdAt":null,"settledAt":null,"expiresAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.invoices().list(&ListParams::default().limit(5)).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].number, 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_list_with_pagination() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/invoices")
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
        .mock("GET", "/v1/invoices/42")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":42,"status":"settled","amount":500,"bolt11":"lnbc5...","reference":null,"memo":null,"preimage":"abc","txNumber":10,"createdAt":null,"settledAt":null,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let inv = client.invoices().get(42).await.unwrap();
    assert_eq!(inv.number, 42);
    assert_eq!(inv.status, InvoiceStatus::Settled);
    assert_eq!(inv.preimage.as_deref(), Some("abc"));
    mock.assert_async().await;
}

#[tokio::test]
async fn invoices_get_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/invoices/abc123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"pending","amount":100,"bolt11":"lnbc1...","reference":null,"memo":null,"preimage":null,"txNumber":null,"createdAt":null,"settledAt":null,"expiresAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let inv = client.invoices().get_by_hash("abc123").await.unwrap();
    assert_eq!(inv.number, 1);
    mock.assert_async().await;
}

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
// Payments
// ---------------------------------------------------------------------------

#[tokio::test]
async fn payments_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/payments")
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
        .mock("GET", "/v1/payments")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":2,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.payments().list(&ListParams::default()).await.unwrap();
    assert_eq!(list.len(), 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_get() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/payments/7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":7,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":1,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let p = client.payments().get(7).await.unwrap();
    assert_eq!(p.number, 7);
    mock.assert_async().await;
}

#[tokio::test]
async fn payments_get_by_hash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/payments/hash123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"number":1,"status":"settled","amount":50,"maxFee":10,"serviceFee":0,"actualFee":null,"address":"a@ln.bot","reference":null,"preimage":null,"txNumber":null,"failureReason":null,"createdAt":null,"settledAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client.payments().get_by_hash("hash123").await.unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Addresses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn addresses_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/addresses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"address":"bot@ln.bot","generated":false,"cost":0,"createdAt":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let addr = client
        .addresses()
        .create(&CreateAddressRequest { address: Some("bot@ln.bot".into()) })
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
        .mock("GET", "/v1/addresses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"address":"a@ln.bot","generated":true,"cost":0,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.addresses().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].generated);
    mock.assert_async().await;
}

#[tokio::test]
async fn addresses_delete_url_encodes() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/addresses/user%40ln.bot")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client.addresses().delete("user@ln.bot").await.unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn addresses_transfer() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/addresses/user%40ln.bot/transfer")
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
        .addresses()
        .transfer("user@ln.bot", &TransferAddressRequest::new("key_other"))
        .await
        .unwrap();
    assert_eq!(resp.address, "user@ln.bot");
    assert_eq!(resp.transferred_to, "w2");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn transactions_list() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/transactions")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"number":1,"type":"credit","amount":100,"balanceAfter":100,"networkFee":0,"serviceFee":0,"paymentHash":null,"preimage":null,"reference":null,"note":null,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client
        .transactions()
        .list(&ListParams::default())
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].tx_type, TransactionType::Credit);
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhooks_create() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/webhooks")
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
        .mock("GET", "/v1/webhooks")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"[{"id":"wh1","url":"https://example.com","active":true,"createdAt":null}]"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let list = client.webhooks().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].active);
    mock.assert_async().await;
}

#[tokio::test]
async fn webhooks_delete() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/v1/webhooks/wh-123")
        .with_status(204)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client.webhooks().delete("wh-123").await.unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Backup
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
// Restore
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
// L402
// ---------------------------------------------------------------------------

#[tokio::test]
async fn l402_create_challenge() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/l402/challenges")
        .match_body(mockito::Matcher::Json(serde_json::json!({"amount": 100})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"macaroon":"mac","invoice":"lnbc1...","paymentHash":"hash","expiresAt":"2024-01-01T00:00:00Z","wwwAuthenticate":"L402 mac:inv"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
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
        .mock("POST", "/v1/l402/verify")
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
        .mock("POST", "/v1/l402/pay")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization":"L402 final","paymentHash":"hash","preimage":"pre","amount":100,"fee":1,"paymentNumber":1,"status":"settled"}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let resp = client
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
