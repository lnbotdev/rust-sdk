use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Account-level wallet operations (create, list).
pub struct WalletsResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl WalletsResource<'_> {
    /// Creates a new wallet.
    pub async fn create(&self) -> Result<CreateWalletResponse, LnBotError> {
        self.client
            .post::<CreateWalletResponse>("/v1/wallets", None::<&()>)
            .await
    }

    /// Lists all wallets for the authenticated user.
    pub async fn list(&self) -> Result<Vec<WalletListItem>, LnBotError> {
        self.client.get("/v1/wallets").await
    }
}
