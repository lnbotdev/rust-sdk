use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Wallet-scoped webhook operations.
pub struct WebhooksResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl WebhooksResource<'_> {
    /// Creates a new webhook.
    pub async fn create(
        &self,
        req: &CreateWebhookRequest,
    ) -> Result<CreateWebhookResponse, LnBotError> {
        self.client
            .post(&format!("{}/webhooks", self.prefix), Some(req))
            .await
    }

    /// Lists all webhooks for the wallet.
    pub async fn list(&self) -> Result<Vec<WebhookResponse>, LnBotError> {
        self.client
            .get(&format!("{}/webhooks", self.prefix))
            .await
    }

    /// Deletes a webhook by ID.
    pub async fn delete(&self, id: &str) -> Result<(), LnBotError> {
        self.client
            .delete(&format!(
                "{}/webhooks/{}",
                self.prefix,
                urlencoding::encode(id)
            ))
            .await
    }
}
