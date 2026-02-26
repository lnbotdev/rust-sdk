use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on API keys.
pub struct KeysResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl KeysResource<'_> {
    /// Lists all API keys for the current wallet.
    pub async fn list(&self) -> Result<Vec<ApiKeyResponse>, LnBotError> {
        self.client.get("/v1/keys").await
    }

    /// Rotates the API key in the given slot (1 = primary, 2 = secondary).
    pub async fn rotate(&self, slot: i32) -> Result<RotateApiKeyResponse, LnBotError> {
        self.client
            .post::<RotateApiKeyResponse>(&format!("/v1/keys/{}/rotate", slot), None::<&()>)
            .await
    }
}
