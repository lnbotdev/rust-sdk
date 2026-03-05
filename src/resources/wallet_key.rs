use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on wallet keys (wk_ keys).
pub struct WalletKeyResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl WalletKeyResource<'_> {
    /// Creates a new wallet key.
    pub async fn create(&self) -> Result<WalletKeyResponse, LnBotError> {
        self.client
            .post::<WalletKeyResponse>(&format!("{}/key", self.prefix), None::<&()>)
            .await
    }

    /// Gets wallet key info.
    pub async fn get(&self) -> Result<WalletKeyInfoResponse, LnBotError> {
        self.client.get(&format!("{}/key", self.prefix)).await
    }

    /// Deletes the wallet key.
    pub async fn delete(&self) -> Result<(), LnBotError> {
        self.client.delete(&format!("{}/key", self.prefix)).await
    }

    /// Rotates the wallet key.
    pub async fn rotate(&self) -> Result<WalletKeyResponse, LnBotError> {
        self.client
            .post::<WalletKeyResponse>(&format!("{}/key/rotate", self.prefix), None::<&()>)
            .await
    }
}
