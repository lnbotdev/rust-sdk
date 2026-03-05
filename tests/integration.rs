/// Integration tests that hit the live API.
/// Run with: LNBOT_USER_KEY=uk_... LNBOT_WALLET_ID=wal_... cargo test -- --ignored
use lnbot::*;
use std::sync::OnceLock;

fn user_key() -> &'static str {
    static KEY: OnceLock<String> = OnceLock::new();
    KEY.get_or_init(|| std::env::var("LNBOT_USER_KEY").expect("LNBOT_USER_KEY must be set"))
}

fn wallet_id() -> &'static str {
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| std::env::var("LNBOT_WALLET_ID").expect("LNBOT_WALLET_ID must be set"))
}

fn client() -> LnBot {
    LnBot::new(user_key())
}

// ---------------------------------------------------------------------------
// Account
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn account_register() {
    let c = LnBot::unauthenticated();
    let resp = c.register().await.unwrap();
    assert!(!resp.user_id.is_empty());
    assert!(resp.primary_key.starts_with("uk_"));
    assert!(resp.secondary_key.starts_with("uk_"));
    assert!(!resp.recovery_passphrase.is_empty());
}

#[tokio::test]
#[ignore]
async fn account_me() {
    let c = client();
    let resp = c.me().await.unwrap();
    assert!(!resp.user_id.is_empty());
}

// ---------------------------------------------------------------------------
// Wallets
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn wallets_create_and_list() {
    let c = client();
    let created = c.wallets().create().await.unwrap();
    assert!(created.wallet_id.starts_with("wal_"));
    assert!(!created.address.is_empty());

    let list = c.wallets().list().await.unwrap();
    assert!(list.iter().any(|w| w.wallet_id == created.wallet_id));
}

#[tokio::test]
#[ignore]
async fn wallet_get_and_update() {
    let c = client();
    let w = c.wallet(wallet_id());
    let info = w.get().await.unwrap();
    assert_eq!(info.wallet_id, wallet_id());

    let updated = w.update(&UpdateWalletRequest::new("integration-test")).await.unwrap();
    assert_eq!(updated.name, "integration-test");
}

// ---------------------------------------------------------------------------
// Wallet Keys
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn wallet_key_lifecycle() {
    let c = client();
    let w = c.wallet(wallet_id());

    // Delete existing key if any
    let _ = w.key().delete().await;

    // Create
    let created = w.key().create().await.unwrap();
    assert!(created.key.starts_with("wk_"));

    // Get info
    let info = w.key().get().await.unwrap();
    assert!(info.hint.starts_with("wk_"));

    // Rotate
    let rotated = w.key().rotate().await.unwrap();
    assert!(rotated.key.starts_with("wk_"));
    assert_ne!(rotated.key, created.key);

    // Delete
    w.key().delete().await.unwrap();
    let err = w.key().get().await.unwrap_err();
    assert!(matches!(err, LnBotError::NotFound { .. }));
}

// ---------------------------------------------------------------------------
// Addresses
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn addresses_lifecycle() {
    let c = client();
    let w = c.wallet(wallet_id());

    // Create random address
    let addr = w.addresses().create(&CreateAddressRequest::default()).await.unwrap();
    assert!(addr.generated);
    assert!(addr.address.ends_with("@ln.bot"));

    // List
    let list = w.addresses().list().await.unwrap();
    assert!(list.iter().any(|a| a.address == addr.address));

    // Delete
    w.addresses().delete(&addr.address).await.unwrap();
}

// ---------------------------------------------------------------------------
// Invoices
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn invoices_create_and_get() {
    let c = client();
    let w = c.wallet(wallet_id());

    let inv = w
        .invoices()
        .create(&CreateInvoiceRequest::new(1000).memo("integration test"))
        .await
        .unwrap();
    assert_eq!(inv.amount, 1000);
    assert_eq!(inv.status, InvoiceStatus::Pending);
    assert!(inv.bolt11.starts_with("lnbc"));

    let fetched = w.invoices().get(inv.number).await.unwrap();
    assert_eq!(fetched.number, inv.number);
    assert_eq!(fetched.amount, 1000);
}

#[tokio::test]
#[ignore]
async fn invoices_list() {
    let c = client();
    let w = c.wallet(wallet_id());
    let list = w.invoices().list(&ListParams::default().limit(5)).await.unwrap();
    assert!(!list.is_empty());
}

// ---------------------------------------------------------------------------
// Public Invoices
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn public_invoice_for_wallet() {
    let c = LnBot::unauthenticated();
    let resp = c
        .invoices()
        .create_for_wallet(&CreateInvoiceForWalletRequest::new(wallet_id(), 1000))
        .await
        .unwrap();
    assert!(resp.bolt11.starts_with("lnbc"));
    assert_eq!(resp.amount, 1000);
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn payments_resolve() {
    let c = client();
    let w = c.wallet(wallet_id());

    // Get a lightning address for this wallet
    let addrs = w.addresses().list().await.unwrap();
    if let Some(addr) = addrs.first() {
        let resolved = w.payments().resolve(&addr.address).await.unwrap();
        assert_eq!(resolved.target, addr.address);
        assert!(!resolved.target_type.is_empty());
    }
}

#[tokio::test]
#[ignore]
async fn payments_list() {
    let c = client();
    let w = c.wallet(wallet_id());
    let _list = w.payments().list(&ListParams::default().limit(5)).await.unwrap();
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn transactions_list() {
    let c = client();
    let w = c.wallet(wallet_id());
    let _list = w.transactions().list(&ListParams::default().limit(5)).await.unwrap();
}

// ---------------------------------------------------------------------------
// Webhooks
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn webhooks_lifecycle() {
    let c = client();
    let w = c.wallet(wallet_id());

    let wh = w
        .webhooks()
        .create(&CreateWebhookRequest::new("https://example.com/integration-test"))
        .await
        .unwrap();
    assert!(!wh.id.is_empty());
    assert!(!wh.secret.is_empty());

    let list = w.webhooks().list().await.unwrap();
    assert!(list.iter().any(|h| h.id == wh.id));

    w.webhooks().delete(&wh.id).await.unwrap();
}

// ---------------------------------------------------------------------------
// L402
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn l402_challenge_and_verify() {
    let c = client();
    let w = c.wallet(wallet_id());

    let challenge = w
        .l402()
        .create_challenge(&CreateL402ChallengeRequest {
            amount: 1,
            description: Some("integration test".into()),
            expiry_seconds: Some(3600),
            caveats: None,
        })
        .await
        .unwrap();
    assert!(!challenge.macaroon.is_empty());
    assert!(!challenge.payment_hash.is_empty());
    assert!(challenge.www_authenticate.starts_with("L402"));
}

// ---------------------------------------------------------------------------
// Backup
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn backup_recovery() {
    let c = client();
    let resp = c.backup().recovery().await.unwrap();
    assert!(!resp.passphrase.is_empty());
}

// ---------------------------------------------------------------------------
// API Keys
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn keys_rotate() {
    // Use secondary key slot to avoid disrupting the primary key we use for tests
    let c = client();
    let resp = c.keys().rotate(1).await.unwrap();
    assert!(!resp.key.is_empty());
    assert!(!resp.name.is_empty());
}

// ---------------------------------------------------------------------------
// Error Handling
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn error_unauthorized() {
    let c = LnBot::new("uk_invalid");
    let err = c.me().await.unwrap_err();
    assert!(matches!(err, LnBotError::Unauthorized { .. }));
}

#[tokio::test]
#[ignore]
async fn error_not_found() {
    let c = client();
    let err = c.wallet(wallet_id()).invoices().get(999999).await.unwrap_err();
    assert!(matches!(err, LnBotError::NotFound { .. }));
}

// ---------------------------------------------------------------------------
// SSE Invoice Watch
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn sse_invoice_watch() {
    use futures_util::StreamExt;

    let c = client();
    let w = c.wallet(wallet_id());

    let inv = w
        .invoices()
        .create(&CreateInvoiceRequest::new(1).memo("sse-test"))
        .await
        .unwrap();

    // Start watching in a spawned task
    let bolt11 = inv.bolt11.clone();
    let invoices = w.invoices();
    let mut stream = invoices.watch(inv.number, Some(30));

    // Spawn a task to pay after a delay
    let c2 = LnBot::new(user_key());
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        c2.wallet(wallet_id())
            .payments()
            .create(&CreatePaymentRequest::new(&bolt11))
            .await
            .unwrap();
    });

    let mut settled = false;
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(15));
    tokio::pin!(timeout);
    loop {
        tokio::select! {
            event = stream.next() => {
                match event {
                    Some(Ok(e)) if e.event == InvoiceEventType::Settled => {
                        settled = true;
                        break;
                    }
                    Some(Ok(_)) => continue,
                    _ => break,
                }
            }
            _ = &mut timeout => break,
        }
    }
    assert!(settled);
}

// ---------------------------------------------------------------------------
// SSE Events Stream
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn sse_events_stream() {
    use futures_util::StreamExt;

    let c = client();
    let w = c.wallet(wallet_id());

    let inv = w
        .invoices()
        .create(&CreateInvoiceRequest::new(1).memo("events-test"))
        .await
        .unwrap();

    let bolt11 = inv.bolt11.clone();
    let events_res = w.events();
    let mut stream = events_res.stream();

    // Spawn a task to pay after a delay
    let c2 = LnBot::new(user_key());
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        c2.wallet(wallet_id())
            .payments()
            .create(&CreatePaymentRequest::new(&bolt11))
            .await
            .unwrap();
    });

    let mut got_event = false;
    let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(15));
    tokio::pin!(timeout);
    loop {
        tokio::select! {
            event = stream.next() => {
                if let Some(Ok(_)) = event {
                    got_event = true;
                    break;
                }
            }
            _ = &mut timeout => break,
        }
    }
    assert!(got_event);
}
