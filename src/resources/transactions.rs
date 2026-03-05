use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Wallet-scoped transaction operations.
pub struct TransactionsResource<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: &'a str,
}

impl TransactionsResource<'_> {
    /// Lists transactions with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<TransactionResponse>, LnBotError> {
        self.client
            .get_with_params(&format!("{}/transactions", self.prefix), params)
            .await
    }
}
