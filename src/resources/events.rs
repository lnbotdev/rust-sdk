use crate::client::{check_status, LnBot};
use crate::errors::LnBotError;
use crate::types::WalletEvent;
use futures_core::Stream;
use futures_util::StreamExt;
use std::pin::Pin;

/// Wallet-scoped real-time event stream.
pub struct EventsResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl EventsResource<'_> {
    /// Opens an SSE stream of all wallet events.
    pub fn stream(
        &self,
    ) -> Pin<Box<dyn Stream<Item = Result<WalletEvent, LnBotError>> + Send + '_>> {
        let url = format!("{}{}/events", self.client.base_url, self.prefix);
        Box::pin(async_stream::try_stream! {
            let mut req = self.client.http.get(&url).header("Accept", "text/event-stream");
            if let Some(ref key) = self.client.api_key {
                req = req.bearer_auth(key);
            }

            let resp = check_status(req.send().await?).await?;

            let mut buffer = String::new();
            let mut stream = resp.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].to_string();
                    buffer.drain(..=pos);

                    if let Some(value) = line.strip_prefix("data:") {
                        let raw = value.trim();
                        if !raw.is_empty() {
                            let event: WalletEvent = serde_json::from_str(raw)?;
                            yield event;
                        }
                    }
                }
            }
        })
    }
}
