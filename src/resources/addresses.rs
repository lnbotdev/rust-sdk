use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on Lightning addresses.
pub struct AddressesResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl AddressesResource<'_> {
    /// Creates or claims a Lightning address.
    pub async fn create(&self, req: &CreateAddressRequest) -> Result<AddressResponse, LnBotError> {
        self.client.post("/v1/addresses", Some(req)).await
    }

    /// Lists all Lightning addresses for the current wallet.
    pub async fn list(&self) -> Result<Vec<AddressResponse>, LnBotError> {
        self.client.get("/v1/addresses").await
    }

    /// Deletes a Lightning address.
    pub async fn delete(&self, address: &str) -> Result<(), LnBotError> {
        self.client
            .delete(&format!("/v1/addresses/{}", urlencoding::encode(address)))
            .await
    }

    /// Transfers a Lightning address to another wallet.
    pub async fn transfer(
        &self,
        address: &str,
        req: &TransferAddressRequest,
    ) -> Result<TransferAddressResponse, LnBotError> {
        self.client
            .post(
                &format!("/v1/addresses/{}/transfer", urlencoding::encode(address)),
                Some(req),
            )
            .await
    }
}
