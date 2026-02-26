# lnbot

[![crates.io](https://img.shields.io/crates/v/lnbot.svg)](https://crates.io/crates/lnbot)
[![docs.rs](https://docs.rs/lnbot/badge.svg)](https://docs.rs/lnbot)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

**The official Rust SDK for [LnBot](https://ln.bot)** — Bitcoin for AI Agents.

Give your AI agents, apps, and services access to Bitcoin over the Lightning Network. Create wallets, send and receive sats, and get real-time payment notifications.

```rust
use lnbot::{LnBot, CreateInvoiceRequest};

let client = LnBot::new("key_...");
let invoice = client.invoices().create(
    &CreateInvoiceRequest::new(1000).memo("Coffee"),
).await?;
```

> LnBot also ships a **[TypeScript SDK](https://www.npmjs.com/package/@lnbot/sdk)**, **[Python SDK](https://pypi.org/project/lnbot/)**, **[Go SDK](https://pkg.go.dev/github.com/lnbotdev/go-sdk)**, **[CLI](https://ln.bot/docs)**, and **[MCP server](https://ln.bot/docs)**.

---

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lnbot = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## Quick start

### Create a wallet

```rust
use lnbot::{LnBot, CreateWalletRequest};

let client = LnBot::unauthenticated();
let wallet = client.wallets().create(&CreateWalletRequest {
    name: Some("my-agent".into()),
}).await?;

println!("{}", wallet.primary_key);
println!("{}", wallet.address);
```

### Receive sats

```rust
use lnbot::{LnBot, CreateInvoiceRequest};

let client = LnBot::new(&wallet.primary_key);
let invoice = client.invoices().create(
    &CreateInvoiceRequest::new(1000).memo("Payment for task #42"),
).await?;
println!("{}", invoice.bolt11);
```

### Wait for payment (SSE)

```rust
use futures_util::StreamExt;
use lnbot::InvoiceEventType;

let mut stream = client.invoices().watch(invoice.number, None);
while let Some(event) = stream.next().await {
    let event = event?;
    if event.event == InvoiceEventType::Settled {
        println!("Paid!");
    }
}
```

### Send sats

```rust
use lnbot::CreatePaymentRequest;

client.payments().create(
    &CreatePaymentRequest::new("alice@ln.bot").amount(500),
).await?;
```

### Check balance

```rust
let wallet = client.wallets().current().await?;
println!("{} sats available", wallet.available);
```

---

## Error handling

```rust
use lnbot::LnBotError;

match client.invoices().get(999).await {
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

let client = LnBot::new("key_...")
    .with_base_url("https://api.ln.bot");
```

---

## Features

- **Async-first** — built on `reqwest` + `tokio`
- **Strongly typed** — every request/response is a Rust struct with `serde` derives
- **Typed enums** — `InvoiceStatus`, `PaymentStatus`, `TransactionType` are real enums, not strings
- **SSE streaming** — `watch` returns a `Stream` of typed events
- **Typed errors** — `LnBotError` enum with `BadRequest`, `NotFound`, `Conflict` variants
- **Forward-compatible** — `#[non_exhaustive]` and `#[serde(other)]` for safe API evolution

## Requirements

- Rust 2021 edition
- Get your API key at [ln.bot](https://ln.bot)

## Links

- [ln.bot](https://ln.bot) — website
- [Documentation](https://ln.bot/docs)
- [GitHub](https://github.com/lnbotdev)
- [docs.rs](https://docs.rs/lnbot)
- [crates.io](https://crates.io/crates/lnbot)

## Other SDKs

- [TypeScript SDK](https://github.com/lnbotdev/typescript-sdk) · [npm](https://www.npmjs.com/package/@lnbot/sdk)
- [Python SDK](https://github.com/lnbotdev/python-sdk) · [pypi](https://pypi.org/project/lnbot/)
- [Go SDK](https://github.com/lnbotdev/go-sdk) · [pkg.go.dev](https://pkg.go.dev/github.com/lnbotdev/go-sdk)

## License

MIT
