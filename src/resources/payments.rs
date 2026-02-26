use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations on payments.
pub struct PaymentsResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl PaymentsResource<'_> {
    /// Creates a new outgoing payment.
    pub async fn create(&self, req: &CreatePaymentRequest) -> Result<PaymentResponse, LnBotError> {
        self.client.post("/v1/payments", Some(req)).await
    }

    /// Lists payments with optional pagination.
    pub async fn list(&self, params: &ListParams) -> Result<Vec<PaymentResponse>, LnBotError> {
        self.client.get_with_params("/v1/payments", params).await
    }

    /// Gets a payment by its number.
    pub async fn get(&self, number: i32) -> Result<PaymentResponse, LnBotError> {
        self.client
            .get(&format!("/v1/payments/{}", number))
            .await
    }
}
