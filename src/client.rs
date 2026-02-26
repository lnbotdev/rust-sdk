use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::{from_status, LnBotError};
use crate::resources::*;
use crate::types::ListParams;

const DEFAULT_BASE_URL: &str = "https://api.ln.bot";

/// LnBot API client.
///
/// Create an instance with [`LnBot::new`] (authenticated) or
/// [`LnBot::unauthenticated`] (for wallet creation), then access resources
/// through accessor methods like [`invoices`](LnBot::invoices),
/// [`payments`](LnBot::payments), etc.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), lnbot::LnBotError> {
/// use lnbot::LnBot;
///
/// let client = LnBot::new("key_...");
/// let wallet = client.wallets().current().await?;
/// println!("{} sats", wallet.available);
/// # Ok(())
/// # }
/// ```
pub struct LnBot {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
}

impl LnBot {
    /// Creates a new authenticated client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: Some(api_key.into()),
        }
    }

    /// Creates a new unauthenticated client for wallet creation.
    pub fn unauthenticated() -> Self {
        Self {
            http: Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: None,
        }
    }

    /// Overrides the base URL for the API.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Overrides the inner [`reqwest::Client`] used for HTTP requests.
    #[must_use]
    pub fn with_http_client(mut self, client: Client) -> Self {
        self.http = client;
        self
    }

    /// Access wallet operations.
    pub fn wallets(&self) -> WalletsResource<'_> {
        WalletsResource { client: self }
    }

    /// Access API key operations.
    pub fn keys(&self) -> KeysResource<'_> {
        KeysResource { client: self }
    }

    /// Access invoice operations.
    pub fn invoices(&self) -> InvoicesResource<'_> {
        InvoicesResource { client: self }
    }

    /// Access payment operations.
    pub fn payments(&self) -> PaymentsResource<'_> {
        PaymentsResource { client: self }
    }

    /// Access Lightning address operations.
    pub fn addresses(&self) -> AddressesResource<'_> {
        AddressesResource { client: self }
    }

    /// Access transaction operations.
    pub fn transactions(&self) -> TransactionsResource<'_> {
        TransactionsResource { client: self }
    }

    /// Access webhook operations.
    pub fn webhooks(&self) -> WebhooksResource<'_> {
        WebhooksResource { client: self }
    }

    /// Access backup operations.
    pub fn backup(&self) -> BackupResource<'_> {
        BackupResource { client: self }
    }

    /// Access restore operations.
    pub fn restore(&self) -> RestoreResource<'_> {
        RestoreResource { client: self }
    }

    pub(crate) fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        let req = req.header("Accept", "application/json");
        match &self.api_key {
            Some(key) => req.bearer_auth(key),
            None => req,
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, LnBotError> {
        let req = self.auth(self.http.get(format!("{}{}", self.base_url, path)));
        handle_json(req.send().await?).await
    }

    pub(crate) async fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &ListParams,
    ) -> Result<T, LnBotError> {
        let mut query: Vec<(&str, String)> = Vec::new();
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(after) = params.after {
            query.push(("after", after.to_string()));
        }
        let req = self
            .auth(self.http.get(format!("{}{}", self.base_url, path)))
            .query(&query);
        handle_json(req.send().await?).await
    }

    pub(crate) async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<T, LnBotError> {
        let mut req = self.auth(self.http.post(format!("{}{}", self.base_url, path)));
        if let Some(b) = body {
            req = req.json(b);
        }
        handle_json(req.send().await?).await
    }

    pub(crate) async fn post_no_response(
        &self,
        path: &str,
        body: Option<&(impl Serialize + ?Sized)>,
    ) -> Result<(), LnBotError> {
        let mut req = self.auth(self.http.post(format!("{}{}", self.base_url, path)));
        if let Some(b) = body {
            req = req.json(b);
        }
        handle_empty(req.send().await?).await
    }

    pub(crate) async fn patch<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<T, LnBotError> {
        let req = self
            .auth(self.http.patch(format!("{}{}", self.base_url, path)))
            .json(body);
        handle_json(req.send().await?).await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<(), LnBotError> {
        let req = self.auth(self.http.delete(format!("{}{}", self.base_url, path)));
        handle_empty(req.send().await?).await
    }
}

async fn handle_json<T: DeserializeOwned>(resp: Response) -> Result<T, LnBotError> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let body = resp.text().await.unwrap_or_default();
        return Err(from_status(status, body));
    }
    Ok(resp.json().await?)
}

async fn handle_empty(resp: Response) -> Result<(), LnBotError> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let body = resp.text().await.unwrap_or_default();
        return Err(from_status(status, body));
    }
    Ok(())
}
