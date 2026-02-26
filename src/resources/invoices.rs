use crate::client::{check_status, LnBot};
use crate::errors::LnBotError;
use crate::types::*;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;

/// Operations on invoices.
pub struct InvoicesResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl InvoicesResource<'_> {
    /// Creates a new invoice.
    pub async fn create(&self, req: &CreateInvoiceRequest) -> Result<InvoiceResponse, LnBotError> {
        self.client.post("/v1/invoices", Some(req)).await
    }

    /// Lists invoices with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<InvoiceResponse>, LnBotError> {
        self.client.get_with_params("/v1/invoices", params).await
    }

    /// Gets an invoice by its number.
    pub async fn get(&self, number: i32) -> Result<InvoiceResponse, LnBotError> {
        self.client
            .get(&format!("/v1/invoices/{}", number))
            .await
    }

    /// Returns a stream of real-time events for an invoice via Server-Sent Events.
    ///
    /// The stream yields [`InvoiceEvent`]s as they arrive. Use this to wait for
    /// an invoice to be settled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example(client: &lnbot::LnBot) -> Result<(), lnbot::LnBotError> {
    /// use futures_util::StreamExt;
    /// use lnbot::InvoiceEventType;
    ///
    /// let mut stream = client.invoices().watch(1, None);
    /// while let Some(event) = stream.next().await {
    ///     let event = event?;
    ///     if event.event == InvoiceEventType::Settled {
    ///         println!("Paid!");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn watch(
        &self,
        number: i32,
        timeout: Option<i32>,
    ) -> Pin<Box<dyn Stream<Item = Result<InvoiceEvent, LnBotError>> + Send + '_>> {
        Box::pin(async_stream::try_stream! {
            let mut url = format!("{}/v1/invoices/{}/events", self.client.base_url, number);
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
                            let data: InvoiceResponse = serde_json::from_str(raw)?;
                            yield InvoiceEvent {
                                event: InvoiceEventType::from(event_type.as_str()),
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
