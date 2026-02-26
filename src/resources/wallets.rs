use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on wallets.
pub struct WalletsResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl WalletsResource<'_> {
    /// Creates a new wallet. Use [`LnBot::unauthenticated`] for this endpoint.
    pub async fn create(&self, req: &CreateWalletRequest) -> Result<CreateWalletResponse, LnBotError> {
        self.client.post("/v1/wallets", Some(req)).await
    }

    /// Returns the current wallet's state.
    pub async fn current(&self) -> Result<WalletResponse, LnBotError> {
        self.client.get("/v1/wallets/current").await
    }

    /// Updates the current wallet.
    pub async fn update(&self, req: &UpdateWalletRequest) -> Result<WalletResponse, LnBotError> {
        self.client.patch("/v1/wallets/current", req).await
    }
}
