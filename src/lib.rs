//! # lnbot
//!
//! Official Rust SDK for [LnBot](https://ln.bot) — Bitcoin for AI Agents.
//!
//! Give your AI agents, apps, and services access to Bitcoin over the Lightning
//! Network. Create wallets, send and receive sats, and get real-time payment
//! notifications.
//!
//! # Quick start
//!
//! ```no_run
//! # async fn example() -> Result<(), lnbot::LnBotError> {
//! use lnbot::{LnBot, CreateInvoiceRequest};
//!
//! let client = LnBot::new("key_...");
//! let invoice = client.invoices().create(&CreateInvoiceRequest::new(1000)).await?;
//! println!("{}", invoice.bolt11);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod errors;
pub mod resources;
pub mod types;

pub use client::LnBot;
pub use errors::LnBotError;
pub use types::*;
