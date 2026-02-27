use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::{CreateL402ChallengeRequest, L402ChallengeResponse, L402PayResponse, PayL402Request, VerifyL402Request, VerifyL402Response};

/// Operations on L402 paywall authentication.
pub struct L402Resource<'a> {
    pub(crate) client: &'a LnBot,
}

impl L402Resource<'_> {
    /// Creates an L402 challenge (invoice + macaroon) for paywall authentication.
    pub async fn create_challenge(&self, req: &CreateL402ChallengeRequest) -> Result<L402ChallengeResponse, LnBotError> {
        self.client.post("/v1/l402/challenges", Some(req)).await
    }

    /// Verifies an L402 authorization token. Stateless — checks signature, preimage, and caveats.
    pub async fn verify(&self, req: &VerifyL402Request) -> Result<VerifyL402Response, LnBotError> {
        self.client.post("/v1/l402/verify", Some(req)).await
    }

    /// Pays an L402 challenge and returns a ready-to-use Authorization header.
    pub async fn pay(&self, req: &PayL402Request) -> Result<L402PayResponse, LnBotError> {
        self.client.post("/v1/l402/pay", Some(req)).await
    }
}
