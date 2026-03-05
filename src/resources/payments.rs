use crate::client::{check_status, LnBot};
use crate::errors::LnBotError;
use crate::types::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;

/// Wallet-scoped payment operations.
pub struct PaymentsResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl PaymentsResource<'_> {
    /// Creates a new outgoing payment.
    pub async fn create(&self, req: &CreatePaymentRequest) -> Result<PaymentResponse, LnBotError> {
        self.client
            .post(&format!("{}/payments", self.prefix), Some(req))
            .await
    }

    /// Lists payments with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<PaymentResponse>, LnBotError> {
        self.client
            .get_with_params(&format!("{}/payments", self.prefix), params)
            .await
    }

    /// Gets a payment by its number.
    pub async fn get(&self, number: i32) -> Result<PaymentResponse, LnBotError> {
        self.client
            .get(&format!("{}/payments/{}", self.prefix, number))
            .await
    }

    /// Returns a specific payment by its payment hash.
    pub async fn get_by_hash(&self, payment_hash: &str) -> Result<PaymentResponse, LnBotError> {
        self.client
            .get(&format!("{}/payments/{}", self.prefix, payment_hash))
            .await
    }

    /// Resolves a payment target (bolt11, lightning address, LNURL).
    pub async fn resolve(&self, target: &str) -> Result<ResolveTargetResponse, LnBotError> {
        self.client
            .get_with_query(
                &format!("{}/payments/resolve", self.prefix),
                &[("target", target)],
            )
            .await
    }

    /// Returns a stream of real-time events for a payment via Server-Sent Events.
    pub fn watch(
        &self,
        number: i32,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<PaymentEvent, LnBotError>> + Send + '_>> {
        let path = format!("{}/payments/{}/events", self.prefix, number);
        self.sse_stream(&path, timeout)
    }

    /// Returns a stream of real-time events for a payment identified by payment hash.
    pub fn watch_by_hash(
        &self,
        payment_hash: &str,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<PaymentEvent, LnBotError>> + Send + '_>> {
        let path = format!("{}/payments/{}/events", self.prefix, payment_hash);
        self.sse_stream(&path, timeout)
    }

    fn sse_stream(
        &self,
        path: &str,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<PaymentEvent, LnBotError>> + Send + '_>> {
        let mut url = format!("{}{}", self.client.base_url, path);
        if let Some(t) = timeout {
            url = format!("{}?timeout={}", url, t);
        }
        Box::pin(async_stream::try_stream! {
            let mut req = self.client.http.get(&url).header("Accept", "text/event-stream");
            if let Some(ref key) = self.client.api_key {
                req = req.bearer_auth(key);
            }

            let resp = check_status(req.send().await?).await?;

            let mut event_type = String::new();
            let mut buffer = String::new();
            let mut stream = resp.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer.drain(..=pos);

                    if let Some(value) = line.strip_prefix("event:") {
                        event_type = value.trim().to_string();
                    } else if let Some(value) = line.strip_prefix("data:") {
                        let raw = value.trim();
                        if !raw.is_empty() && !event_type.is_empty() {
                            let data: PaymentResponse = serde_json::from_str(raw)?;
                            yield PaymentEvent {
                                event: PaymentEventType::from(event_type.as_str()),
                                data,
                            };
                            event_type.clear();
                        }
                    }
                }
            }
        })
    }
}
