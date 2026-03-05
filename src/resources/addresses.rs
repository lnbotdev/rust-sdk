use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Wallet-scoped Lightning address operations.
pub struct AddressesResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl AddressesResource<'_> {
    /// Creates or claims a Lightning address.
    pub async fn create(&self, req: &CreateAddressRequest) -> Result<AddressResponse, LnBotError> {
        self.client
            .post(&format!("{}/addresses", self.prefix), Some(req))
            .await
    }

    /// Lists all Lightning addresses for the wallet.
    pub async fn list(&self) -> Result<Vec<AddressResponse>, LnBotError> {
        self.client
            .get(&format!("{}/addresses", self.prefix))
            .await
    }

    /// Deletes a Lightning address.
    pub async fn delete(&self, address: &str) -> Result<(), LnBotError> {
        self.client
            .delete(&format!(
                "{}/addresses/{}",
                self.prefix,
                urlencoding::encode(address)
            ))
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
                &format!(
                    "{}/addresses/{}/transfer",
                    self.prefix,
                    urlencoding::encode(address)
                ),
                Some(req),
            )
            .await
    }
}
