use crate::client::{check_status, LnBot};
use crate::errors::LnBotError;
use crate::types::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;

/// Operations on payments.
pub struct PaymentsResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl PaymentsResource<'_> {
    /// Creates a new outgoing payment.
    pub async fn create(&self, req: &CreatePaymentRequest) -> Result<PaymentResponse, LnBotError> {
        self.client.post("/v1/payments", Some(req)).await
    }

    /// Lists payments with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<PaymentResponse>, LnBotError> {
        self.client.get_with_params("/v1/payments", params).await
    }

    /// Gets a payment by its number.
    pub async fn get(&self, number: i32) -> Result<PaymentResponse, LnBotError> {
        self.client
            .get(&format!("/v1/payments/{}", number))
            .await
    }

    /// Returns a specific payment by its payment hash.
    pub async fn get_by_hash(&self, payment_hash: &str) -> Result<PaymentResponse, LnBotError> {
        self.client
            .get(&format!("/v1/payments/{}", payment_hash))
            .await
    }

    /// Returns a stream of real-time events for a payment via Server-Sent Events.
    ///
    /// The stream yields [`PaymentEvent`]s as they arrive. Use this to wait for
    /// a payment to settle or fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(client: &lnbot::LnBot) -> Result<(), lnbot::LnBotError> {
    /// use futures_util::StreamExt;
    /// use lnbot::PaymentEventType;
    ///
    /// let mut stream = client.payments().watch(1, None);
    /// while let Some(event) = stream.next().await {
    ///     let event = event?;
    ///     if event.event == PaymentEventType::Settled {
    ///         println!("Payment complete!");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn watch(
        &self,
        number: i32,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<PaymentEvent, LnBotError>> + Send + '_>> {
        Box::pin(async_stream::try_stream! {
            let mut url = format!("{}/v1/payments/{}/events", self.client.base_url, number);
            if let Some(t) = timeout {
                url = format!("{}?timeout={}", url, t);
            }

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

    /// Returns a stream of real-time events for a payment identified by payment hash
    /// via Server-Sent Events.
    ///
    /// The stream yields [`PaymentEvent`]s as they arrive. Use this to wait for
    /// a payment to settle or fail.
    pub fn watch_by_hash(
        &self,
        payment_hash: &str,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<PaymentEvent, LnBotError>> + Send + '_>> {
        let payment_hash = payment_hash.to_string();
        Box::pin(async_stream::try_stream! {
            let mut url = format!("{}/v1/payments/{}/events", self.client.base_url, payment_hash);
            if let Some(t) = timeout {
                url = format!("{}?timeout={}", url, t);
            }

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
