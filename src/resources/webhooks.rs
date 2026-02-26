use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on webhooks.
pub struct WebhooksResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl WebhooksResource<'_> {
    /// Creates a new webhook.
    pub async fn create(&self, req: &CreateWebhookRequest) -> Result<CreateWebhookResponse, LnBotError> {
        self.client.post("/v1/webhooks", Some(req)).await
    }

    /// Lists all webhooks for the current wallet.
    pub async fn list(&self) -> Result<Vec<WebhookResponse>, LnBotError> {
        self.client.get("/v1/webhooks").await
    }

    /// Deletes a webhook by ID.
    pub async fn delete(&self, id: &str) -> Result<(), LnBotError> {
        self.client
            .delete(&format!("/v1/webhooks/{}", urlencoding::encode(id)))
            .await
    }
}
