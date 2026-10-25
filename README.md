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

An `expired` event is not always final: an invoice paid just before it expired can still turn `settled` shortly after. Before giving up on it, check again with `w.invoices().get(invoice.number)`, or keep listening for `invoice.settled` on the wallet event stream (`w.events().stream()`) or a webhook.

When the wait runs out first (the `timeout` argument, default 60 seconds, max 300), the stream yields a `Timeout` event whose `data` is the current state, then ends: read it again or watch again. `payments().watch` works the same way, with `Settled`, `Failed` or `Timeout`.

Event and webhook payloads use snake_case keys on the wire (`max_fee`, `created_at`). The SDK converts event payloads to the REST field names, so the `data` of a `w.events().stream()` event decodes with `serde_json::from_value` into `InvoiceResponse` or `PaymentResponse`; webhook bodies are not parsed by the SDK and keep the snake_case keys.

### Send sats

```rust
use lnbot::CreatePaymentRequest;

w.payments().create(
    &CreatePaymentRequest::new("alice@ln.bot").amount(500),
).await?;
```

Leave `max_fee` unset. The server caps the network fee (0.25% plus Lightning routing) at 1% of the amount, minimum 20 sats, and holds only the fee estimate plus a small buffer while the payment is in flight. On success the wallet is debited `amount + actual_fee + service_fee`, where `service_fee` is 0.25% (minimum 1 sat). Both fees are 0 between ln.bot wallets. Set `.max_fee(n)` only to allow a higher fee: it accepts 1 up to the larger of 10,000 sats and 5% of the amount.

A payment that can't be sent fails quickly, and its funds are released at once: within about half a minute if the Lightning service refuses it, or after 10 minutes of automatic retries for a temporary problem (`PaymentStage::Retrying`, until `retry_until`). If its invoice expires first, it fails with `Invoice expired before it could be paid.` A payment stays `processing` longer only when its funds are already on the Lightning network: it settles or fails as soon as the network does, usually within seconds, and is never refunded while it could still be paid. Show `failure_reason` as is; when it ends in "Please try again.", sending the payment again later can work. Use `w.payments().watch(number, None)` or the wallet event stream to follow it. Its `stage` and `status_detail` say what is happening meanwhile: see [Payment stages](#payment-stages).

### Check balance

```rust
let info = w.get().await?;
println!("{} sats available", info.available);
```

---

## Pricing a payment

`resolve` and `resolve_with_amount` look up a target and price the payment without sending or holding anything.

### Show fees before paying

```rust
let quote = w.payments().resolve_with_amount("alice@ln.bot", 5_000).await?;
if let (Some(fee), Some(service), Some(total)) =
    (quote.network_fee_estimate, quote.service_fee, quote.total)
{
    println!("network fee ~{fee} sats + service fee {service} sats = {total} sats");
}
if let Some(warning) = &quote.fee_warning {
    eprintln!("{warning}");
}
```

A BOLT11 invoice already carries its amount, so `w.payments().resolve(bolt11)` prices it. Pricing a Lightning address or LNURL requests an invoice from the recipient, so it is slower: call it once the amount is settled, not on every keystroke. When `fee_warning` is set, creating the payment without `max_fee` would be rejected because the fee is unusually high.

### Send everything

```rust
use lnbot::CreatePaymentRequest;

let quote = w.payments().resolve("alice@ln.bot").await?;
if let Some(amount) = quote.max_sendable.filter(|&a| a > 0) {
    w.payments().create(
        &CreatePaymentRequest::new("alice@ln.bot").amount(amount),
    ).await?;
}
```

Resolved without an amount, a Lightning address or LNURL returns `max_sendable`: the most this wallet can send there now, after fees. Pay that amount without `max_fee`, since it was computed with the default fee cap. It is 0 when the balance can't cover the recipient's minimum.

Both calls return `LnBotError::BadRequest` when the target can't be paid. The body is `{ "error": "..." }`, plus `min` and `max` when the amount is outside the recipient's range. An unknown ln.bot address gives `Recipient not found.` and the wallet's own address gives `Cannot transfer to yourself.`

---

## Payment stages

Every payment returned by `create`, `get` and `list` carries `stage`, `status_detail` and `retry_until`. `stage` says where the payment is, more precisely than `status`, and `status_detail` is a sentence to show your user as is.

| `stage` | With `status` | `status_detail` |
|---------|---------------|-----------------|
| `Queued` (`queued`) | `Pending` | Waiting to be sent. |
| `Retrying` (`retrying`) | `Pending` | Not sent yet; retrying for a few minutes. If it still can't be sent, it fails and the funds are released. |
| `Sending` (`sending`) | `Pending` or `Processing` | Being sent. Between ln.bot wallets: Being transferred to the recipient's wallet. |
| `InFlight` (`in_flight`) | `Processing` | Sent; the Lightning network is completing it, usually within seconds. |
| `Confirming` (`confirming`) | `Processing` | Checking with the Lightning network whether it went out. The funds stay on hold until then. |
| `OnHold` (`on_hold`) | `Processing` | Interrupted by an earlier version of the service and on hold until it is reviewed. The funds are held, not lost. |
| `Settled` (`settled`) | `Settled` | `None` |
| `Failed` (`failed`) | `Failed` | The `failure_reason` |

```rust
use lnbot::PaymentStage;

let p = w.payments().get(number).await?;
if let Some(detail) = &p.status_detail {
    println!("{detail}");
}
if p.stage == Some(PaymentStage::Retrying) {
    println!("retried until {}", p.retry_until.as_deref().unwrap_or("soon"));
}
```

More stages may be added. One this SDK doesn't know is `PaymentStage::Unknown`, and an API that doesn't send the field gives `None`: in both cases, go by `status`. A payment can go from `Processing` back to `Pending` when it is retried, so treat both as in progress. `retry_until` is only set while the stage is `Retrying`: if the payment still can't be sent by then, it fails within about a minute and the funds are released.

The events of `w.payments().watch` carry the same fields, but the stream sends nothing when only the stage changes: it ends with `Settled`, `Failed` or `Timeout`. Webhooks and the wallet event stream don't carry a stage.

---

## Service status

Two public calls say whether ln.bot is working, measured about once a minute. They need no API key (sending one does no harm), so an unauthenticated client works.

```rust
use lnbot::{LnBot, ServiceState};

let client = LnBot::unauthenticated();

let status = client.status().await?;
if status.status != ServiceState::Operational {
    println!("ln.bot is {}", status.status);
}
if let Some(notice) = &status.notice {
    println!("{}", notice.message);
}
println!("{} payments in progress", status.components.sending.in_progress);

let history = client.status_history("7d").await?;
for bucket in history.buckets.iter().filter(|b| b.samples > 0) {
    let up = bucket.samples - bucket.overall.degraded - bucket.overall.outage;
    println!("{} {:.1}%", bucket.at, 100.0 * up as f64 / bucket.samples as f64);
}
```

`status` is `Operational`, `Degraded`, `Outage` or `Unknown` (nothing measured for 5 minutes, so the numbers are as old as `updated_at`), overall and for each part in `components`: `sending` (payments out), `receiving` (payments in), `node` (the Lightning node) and `webhooks` (deliveries). `sending.by_stage` counts the payments in progress by [stage](#payment-stages), keyed as the API sends it (`in_flight`), and leaves out stages with none. A notice from the operators has a `level` of `Info`, `Warning` or `Outage` and a `message` to show as is. More states and levels may be added: one this SDK doesn't know comes back as `ServiceState::Other` or `NoticeLevel::Other`, with the value as sent.

`status_history` takes `"24h"` (5-minute buckets, the API's default), `"7d"` (hourly), `"30d"` (6-hourly) or `"90d"` (daily); any other value returns `LnBotError::BadRequest`. Buckets are aligned to UTC and the last one is still filling, so don't assume how many there are. A bucket with `samples == 0` wasn't measured, and its percentiles and gauges are `None`: treat it as no data, not as up or down. `summary` covers the whole range, with `uptime` in percent of measured time.

Both calls share the API's limit for public routes, 10 requests a minute per IP. Over it they return `LnBotError::Api { status: 429, body }`, with the API's message in `body`.

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
client.status()                        // Service status now (no key needed)
client.status_history("24h")           // Service status over time (no key needed)
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

Other statuses come back as `LnBotError::Api { status, body }`, such as 429 when a rate limit is hit.

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
- **Typed enums** -- `InvoiceStatus`, `PaymentStatus`, `PaymentStage`, `TransactionType`, `ServiceState` are real enums, not strings
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
