# ln.bot

[![crates.io](https://img.shields.io/crates/v/lnbot.svg)](https://crates.io/crates/lnbot)
[![docs.rs](https://docs.rs/lnbot/badge.svg)](https://docs.rs/lnbot)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

**The official Rust SDK for [ln.bot](https://ln.bot)** -- Bitcoin for AI Agents.

Give your AI agents, apps, and services access to Bitcoin over the Lightning Network. Create wallets, send and receive sats, and get real-time payment notifications.

```rust
use lnbot::{LnBot, CreateInvoiceRequest};

let client = LnBot::new("uk_...");
let w = client.wallet("wal_...");
let invoice = w.invoices().create(
    &CreateInvoiceRequest::new(1000).memo("Coffee"),
).await?;
```

> ln.bot also ships a **[TypeScript SDK](https://www.npmjs.com/package/@lnbot/sdk)**, **[Python SDK](https://pypi.org/project/lnbot/)**, **[C# SDK](https://www.nuget.org/packages/LnBot)**, **[Go SDK](https://pkg.go.dev/github.com/lnbotdev/go-sdk)**, **[CLI](https://ln.bot/docs)**, and **[MCP server](https://ln.bot/docs)**.

---

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lnbot = "1"
tokio = { version = "1", features = ["full"] }
```

---

## Quick start

### Register an account

```rust
use lnbot::LnBot;

let client = LnBot::unauthenticated();
let account = client.register().await?;
println!("{}", account.primary_key);
println!("{}", account.recovery_passphrase);
```

### Create a wallet

```rust
let client = LnBot::new(&account.primary_key);
let wallet = client.wallets().create().await?;
println!("{}", wallet.wallet_id);
```

### Receive sats

```rust
use lnbot::CreateInvoiceRequest;

let w = client.wallet(&wallet.wallet_id);
let invoice = w.invoices().create(
    &CreateInvoiceRequest::new(1000).memo("Payment for task #42"),
).await?;
println!("{}", invoice.bolt11);
```

### Wait for payment (SSE)

```rust
use futures_util::StreamExt;
use lnbot::InvoiceEventType;

let mut stream = w.invoices().watch(invoice.number, None);
while let Some(event) = stream.next().await {
    let event = event?;
    if event.event == InvoiceEventType::Settled {
        println!("Paid!");
        break;
    }
}
```

### Send sats

```rust
use lnbot::CreatePaymentRequest;

w.payments().create(
    &CreatePaymentRequest::new("alice@ln.bot").amount(500),
).await?;
```

### Check balance

```rust
let info = w.get().await?;
println!("{} sats available", info.available);
```

---

## Wallet-scoped API

All wallet operations go through a `Wallet` handle obtained via `client.wallet(wallet_id)`:

```rust
let w = client.wallet("wal_abc123");

// Wallet info
let info = w.get().await?;
w.update(&UpdateWalletRequest::new("production")).await?;

// Sub-resources
w.key()           // Wallet key management (wk_ keys)
w.invoices()      // Create, list, get, watch invoices
w.payments()      // Send, list, get, resolve, watch payments
w.addresses()     // Create, list, delete, transfer Lightning addresses
w.transactions()  // List transaction history
w.webhooks()      // Create, list, delete webhook endpoints
w.events()        // Real-time SSE event stream
w.l402()          // L402 paywall authentication
```

Account-level operations stay on the client:

```rust
client.register()                      // Register new account
client.me()                            // Get authenticated identity
client.wallets().create()              // Create wallet
client.wallets().list()                // List wallets
client.keys().rotate(0)                // Rotate account key
client.invoices().create_for_wallet()  // Public invoice by wallet ID
client.invoices().create_for_address() // Public invoice by address
```

---

## L402 paywalls

```rust
use lnbot::{CreateL402ChallengeRequest, PayL402Request, VerifyL402Request};

let w = client.wallet("wal_...");

// Create a challenge (server side)
let challenge = w.l402().create_challenge(&CreateL402ChallengeRequest {
    amount: 100,
    description: Some("API access".into()),
    expiry_seconds: Some(3600),
    caveats: None,
}).await?;

// Pay the challenge (client side)
let result = w.l402().pay(&PayL402Request {
    www_authenticate: challenge.www_authenticate,
    max_fee: None, reference: None, wait: None, timeout: None,
}).await?;

// Verify a token (server side, stateless)
let v = w.l402().verify(&VerifyL402Request {
    authorization: result.authorization.unwrap(),
}).await?;
println!("{}", v.valid);
```

## Error handling

```rust
use lnbot::LnBotError;

match w.invoices().get(999).await {
    Ok(invoice) => println!("{:?}", invoice),
    Err(LnBotError::NotFound { body }) => eprintln!("not found: {}", body),
    Err(LnBotError::BadRequest { body }) => eprintln!("bad request: {}", body),
    Err(LnBotError::Conflict { body }) => eprintln!("conflict: {}", body),
    Err(e) => eprintln!("error: {}", e),
}
```

## Configuration

```rust
use lnbot::LnBot;

let client = LnBot::new("uk_...")
    .with_base_url("https://api.ln.bot");
```

---

## Features

- **Async-first** -- built on `reqwest` + `tokio`
- **Wallet-scoped API** -- `client.wallet(id)` returns a typed scope with all sub-resources
- **Strongly typed** -- every request/response is a Rust struct with `serde` derives
- **Typed enums** -- `InvoiceStatus`, `PaymentStatus`, `TransactionType` are real enums, not strings
- **SSE streaming** -- `watch` returns a `Stream` of typed events
- **Typed errors** -- `LnBotError` enum with `BadRequest`, `NotFound`, `Conflict` variants
- **Forward-compatible** -- `#[non_exhaustive]` and `#[serde(other)]` for safe API evolution

## Requirements

- Rust 2021 edition
- Get your API key at [ln.bot](https://ln.bot)

## Links

- [ln.bot](https://ln.bot) -- website
- [Documentation](https://ln.bot/docs)
- [GitHub](https://github.com/lnbotdev)
- [docs.rs](https://docs.rs/lnbot)
- [crates.io](https://crates.io/crates/lnbot)

## Other SDKs

- [TypeScript SDK](https://github.com/lnbotdev/typescript-sdk) . [npm](https://www.npmjs.com/package/@lnbot/sdk)
- [Python SDK](https://github.com/lnbotdev/python-sdk) . [pypi](https://pypi.org/project/lnbot/)
- [C# SDK](https://github.com/lnbotdev/csharp-sdk) . [NuGet](https://www.nuget.org/packages/LnBot)
- [Go SDK](https://github.com/lnbotdev/go-sdk) . [pkg.go.dev](https://pkg.go.dev/github.com/lnbotdev/go-sdk)

## License

MIT
