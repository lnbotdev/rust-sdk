use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on transactions.
pub struct TransactionsResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl TransactionsResource<'_> {
    /// Lists transactions with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<TransactionResponse>, LnBotError> {
        self.client
            .get_with_params("/v1/transactions", params)
            .await
    }
}
