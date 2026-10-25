use lnbot::{LnBot, LnBotError, ServiceState};

// ---------------------------------------------------------------------------
// Client construction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn new_sends_authorization_header() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1")
        .match_header("authorization", "Bearer key_test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"Test","balance":0,"onHold":0,"available":0}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client.wallet("wal_1").get().await.unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn unauthenticated_omits_authorization() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/register")
        .match_header("authorization", mockito::Matcher::Missing)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"userId":"u1","primaryKey":"uk_1","secondaryKey":"uk_2","recoveryPassphrase":"words"}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    client.register().await.unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn sends_accept_json_header() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1")
        .match_header("accept", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"Test","balance":0,"onHold":0,"available":0}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    client.wallet("wal_1").get().await.unwrap();
    mock.assert_async().await;
}

#[tokio::test]
async fn with_base_url_trims_trailing_slash() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"Test","balance":0,"onHold":0,"available":0}"#)
        .create_async()
        .await;

    let url = format!("{}/", server.url());
    let client = LnBot::new("key_test").with_base_url(url);
    client.wallet("wal_1").get().await.unwrap();
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Register / Me
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/register")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"userId":"u1","primaryKey":"uk_1","secondaryKey":"uk_2","recoveryPassphrase":"word1 word2"}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let resp = client.register().await.unwrap();
    assert_eq!(resp.user_id, "u1");
    assert_eq!(resp.primary_key, "uk_1");
    assert_eq!(resp.secondary_key, "uk_2");
    assert_eq!(resp.recovery_passphrase, "word1 word2");
    mock.assert_async().await;
}

#[tokio::test]
async fn me() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"userId":"u1","keyName":"primary","walletId":null}"#)
        .create_async()
        .await;

    let client = LnBot::new("uk_1").with_base_url(server.url());
    let resp = client.me().await.unwrap();
    assert_eq!(resp.user_id, "u1");
    assert_eq!(resp.key_name.as_deref(), Some("primary"));
    assert!(resp.wallet_id.is_none());
    mock.assert_async().await;
}

#[tokio::test]
async fn me_with_wallet_key() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"userId":"u1","keyName":"wallet","walletId":"wal_1"}"#)
        .create_async()
        .await;

    let client = LnBot::new("wk_1").with_base_url(server.url());
    let resp = client.me().await.unwrap();
    assert_eq!(resp.wallet_id.as_deref(), Some("wal_1"));
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Service status
// ---------------------------------------------------------------------------

const STATUS_LIVE: &str = include_str!("fixtures/status_live.json");
const HISTORY_24H_LIVE: &str = include_str!("fixtures/status_history_24h.json");
const HISTORY_90D: &str = include_str!("fixtures/status_history_90d.json");
const TOO_MANY_REQUESTS: &str = r#"{"error":"Too many requests. Please try again later."}"#;

#[tokio::test]
async fn status_without_api_key() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/status")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_header("accept", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("cache-control", "public, max-age=15")
        .with_body(STATUS_LIVE)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let status = client.status().await.unwrap();
    assert_eq!(status.status, ServiceState::Operational);
    assert_eq!(status.components.node.latency_ms, Some(885));
    mock.assert_async().await;
}

#[tokio::test]
async fn status_history_without_api_key() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/status/history")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_query(mockito::Matcher::UrlEncoded("range".into(), "24h".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(HISTORY_24H_LIVE)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let history = client.status_history("24h").await.unwrap();
    assert_eq!(history.range, "24h");
    assert_eq!(history.buckets.len(), 289);
    mock.assert_async().await;
}

#[tokio::test]
async fn status_history_sends_range() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/status/history")
        .match_query(mockito::Matcher::Regex("^range=90d$".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(HISTORY_90D)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let history = client.status_history("90d").await.unwrap();
    assert_eq!(history.bucket_seconds, 86400);
    assert_eq!(history.buckets.len(), 3);
    assert_eq!(history.buckets[2].samples, 0);
    mock.assert_async().await;
}

#[tokio::test]
async fn status_history_passes_any_range_through() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/status/history")
        .match_query(mockito::Matcher::UrlEncoded(
            "range".into(),
            "1 year&more".into(),
        ))
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"range must be 24h, 7d, 30d or 90d."}"#)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let err = client.status_history("1 year&more").await.unwrap_err();
    match err {
        LnBotError::BadRequest { body } => {
            assert!(body.contains("range must be 24h, 7d, 30d or 90d."))
        }
        other => panic!("expected BadRequest, got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn status_calls_surface_429() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/status")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(TOO_MANY_REQUESTS)
        .create_async()
        .await;
    server
        .mock("GET", "/v1/status/history")
        .match_query(mockito::Matcher::Any)
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(TOO_MANY_REQUESTS)
        .create_async()
        .await;

    let client = LnBot::unauthenticated().with_base_url(server.url());
    let errors = [
        client.status().await.unwrap_err(),
        client.status_history("7d").await.unwrap_err(),
    ];
    for err in errors {
        assert!(err
            .to_string()
            .contains("Too many requests. Please try again later."));
        match err {
            LnBotError::Api { status, body } => {
                assert_eq!(status, 429);
                assert_eq!(body, TOO_MANY_REQUESTS);
            }
            other => panic!("expected Api, got {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// Wallet factory
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "wallet_id must not be empty")]
fn wallet_rejects_empty_id() {
    let client = LnBot::new("key_test");
    client.wallet("");
}

#[tokio::test]
async fn wallet_get() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"My Wallet","balance":1000,"onHold":100,"available":900}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let w = client.wallet("wal_1").get().await.unwrap();
    assert_eq!(w.wallet_id, "wal_1");
    assert_eq!(w.balance, 1000);
    assert_eq!(w.available, 900);
    mock.assert_async().await;
}

#[tokio::test]
async fn wallet_update() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PATCH", "/v1/wallets/wal_1")
        .match_body(mockito::Matcher::Json(serde_json::json!({"name": "Renamed"})))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"walletId":"wal_1","name":"Renamed","balance":0,"onHold":0,"available":0}"#)
        .create_async()
        .await;

    let client = LnBot::new("key_test").with_base_url(server.url());
    let w = client
        .wallet("wal_1")
        .update(&lnbot::UpdateWalletRequest::new("Renamed"))
        .await
        .unwrap();
    assert_eq!(w.name, "Renamed");
    mock.assert_async().await;
}

// ---------------------------------------------------------------------------
// Error mapping from HTTP status codes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn maps_400_to_bad_request() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(400)
        .with_body(r#"{"message":"bad"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::BadRequest { .. }));
}

#[tokio::test]
async fn maps_401_to_unauthorized() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(401)
        .with_body(r#"{"message":"unauthorized"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::Unauthorized { .. }));
}

#[tokio::test]
async fn maps_403_to_forbidden() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(403)
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::Forbidden { .. }));
}

#[tokio::test]
async fn maps_404_to_not_found() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(404)
        .with_body(r#"{"message":"not found"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::NotFound { .. }));
}

#[tokio::test]
async fn maps_409_to_conflict() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(409)
        .with_body(r#"{"message":"conflict"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::Conflict { .. }));
}

#[tokio::test]
async fn maps_500_to_api_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(500)
        .with_body("internal error")
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    match err {
        LnBotError::Api { status, body } => {
            assert_eq!(status, 500);
            assert_eq!(body, "internal error");
        }
        other => panic!("expected Api, got {:?}", other),
    }
}

#[tokio::test]
async fn error_preserves_response_body() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(401)
        .with_body(r#"{"message":"invalid key"}"#)
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("401"));
    assert!(msg.contains("invalid key"));
}

// ---------------------------------------------------------------------------
// JSON deserialization error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn malformed_json_returns_http_error() {
    let mut server = mockito::Server::new_async().await;
    server
        .mock("GET", "/v1/wallets/wal_1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not json")
        .create_async()
        .await;

    let client = LnBot::new("k").with_base_url(server.url());
    let err = client.wallet("wal_1").get().await.unwrap_err();
    assert!(matches!(err, LnBotError::Http(_)));
}
