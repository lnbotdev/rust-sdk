use crate::client::LnBot;
use crate::errors::LnBotError;
use crate::types::*;

/// Operations for backing up wallet credentials.
pub struct BackupResource<'a> {
    pub(crate) client: &'a LnBot,
}

impl BackupResource<'_> {
    /// Returns the wallet's recovery passphrase.
    pub async fn recovery(&self) -> Result<RecoveryBackupResponse, LnBotError> {
        self.client
            .post::<RecoveryBackupResponse>("/v1/backup/recovery", None::<&()>)
            .await
    }

    /// Begins a passkey backup flow.
    pub async fn passkey_begin(&self) -> Result<BackupPasskeyBeginResponse, LnBotError> {
        self.client
            .post::<BackupPasskeyBeginResponse>("/v1/backup/passkey/begin", None::<&()>)
            .await
    }

    /// Completes a passkey backup flow.
    pub async fn passkey_complete(
        &self,
        req: &BackupPasskeyCompleteRequest,
    ) -> Result<(), LnBotError> {
        self.client
            .post_no_response("/v1/backup/passkey/complete", Some(req))
            .await
    }
}
