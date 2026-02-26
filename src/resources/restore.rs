use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations for restoring wallet credentials.
pub struct RestoreResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl RestoreResource<'_> {
    /// Restores a wallet from a recovery passphrase.
    pub async fn recovery(
        &self,
        req: &RecoveryRestoreRequest,
    ) -> Result<RecoveryRestoreResponse, LnBotError> {
        self.client.post("/v1/restore/recovery", Some(req)).await
    }

    /// Begins a passkey restore flow.
    pub async fn passkey_begin(&self) -> Result<RestorePasskeyBeginResponse, LnBotError> {
        self.client
            .post::<RestorePasskeyBeginResponse>("/v1/restore/passkey/begin", None::<&()>)
            .await
    }

    /// Completes a passkey restore flow.
    pub async fn passkey_complete(
        &self,
        req: &RestorePasskeyCompleteRequest,
    ) -> Result<RestorePasskeyCompleteResponse, LnBotError> {
        self.client
            .post("/v1/restore/passkey/complete", Some(req))
            .await
    }
}
