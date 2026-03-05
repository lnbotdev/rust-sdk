use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::{from_status, LnBotError};
use crate::resources::*;
use crate::types::*;

const DEFAULT_BASE_URL: &str = "https://api.ln.bot";

/// LnBot API client.
///
/// Create an instance with [`LnBot::new`] (authenticated) or
/// [`LnBot::unauthenticated`] (for public endpoints), then access
/// wallet-scoped resources through [`wallet`](LnBot::wallet).
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), lnbot::LnBotError> {
/// use lnbot::{LnBot, CreateInvoiceRequest};
///
/// let client = LnBot::new("uk_...");
/// let w = client.wallet("wal_...");
/// let invoice = w.invoices().create(&CreateInvoiceRequest::new(1000)).await?;
/// println!("{}", invoice.bolt11);
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

    /// Creates a new unauthenticated client for public endpoints.
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

    /// Registers a new account. No authentication required.
    pub async fn register(&self) -> Result<RegisterResponse, LnBotError> {
        self.post("/v1/register", None::<&()>).await
    }

    /// Returns the authenticated identity.
    pub async fn me(&self) -> Result<MeResponse, LnBotError> {
        self.get("/v1/me").await
    }

    /// Returns a wallet handle for the given wallet ID.
    ///
    /// All wallet-scoped operations go through this handle.
    pub fn wallet(&self, wallet_id: &str) -> Wallet<'_> {
        assert!(!wallet_id.is_empty(), "wallet_id must not be empty");
        Wallet {
            client: self,
            prefix: format!("/v1/wallets/{}", wallet_id),
        }
    }

    /// Access wallet operations (create, list).
    pub fn wallets(&self) -> WalletsResource<'_> {
        WalletsResource { client: self }
    }

    /// Access API key operations.
    pub fn keys(&self) -> KeysResource<'_> {
        KeysResource { client: self }
    }

    /// Access public invoice operations (no auth required).
    pub fn invoices(&self) -> PublicInvoicesResource<'_> {
        PublicInvoicesResource { client: self }
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

    pub(crate) async fn get_with_query<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, LnBotError> {
        let req = self
            .auth(self.http.get(format!("{}{}", self.base_url, path)))
            .query(query);
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

/// A wallet handle. All wallet-scoped operations go through this.
pub struct Wallet<'a> {
    pub(crate) client: &'a LnBot,
    pub(crate) prefix: String,
}

impl<'a> Wallet<'a> {
    /// Returns the wallet's current state.
    pub async fn get(&self) -> Result<WalletResponse, LnBotError> {
        self.client.get(&self.prefix).await
    }

    /// Updates the wallet.
    pub async fn update(&self, req: &UpdateWalletRequest) -> Result<WalletResponse, LnBotError> {
        self.client.patch(&self.prefix, req).await
    }

    /// Access wallet key operations.
    pub fn key(&self) -> WalletKeyResource<'_> {
        WalletKeyResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access invoice operations.
    pub fn invoices(&self) -> InvoicesResource<'_> {
        InvoicesResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access payment operations.
    pub fn payments(&self) -> PaymentsResource<'_> {
        PaymentsResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access Lightning address operations.
    pub fn addresses(&self) -> AddressesResource<'_> {
        AddressesResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access transaction operations.
    pub fn transactions(&self) -> TransactionsResource<'_> {
        TransactionsResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access webhook operations.
    pub fn webhooks(&self) -> WebhooksResource<'_> {
        WebhooksResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access the real-time wallet event stream.
    pub fn events(&self) -> EventsResource<'_> {
        EventsResource {
            client: self.client,
            prefix: &self.prefix,
        }
    }

    /// Access L402 paywall operations.
    pub fn l402(&self) -> L402Resource<'_> {
        L402Resource {
            client: self.client,
            prefix: &self.prefix,
        }
    }
}

pub(crate) async fn check_status(resp: Response) -> Result<Response, LnBotError> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let body = resp.text().await.unwrap_or_default();
        return Err(from_status(status, body));
    }
    Ok(resp)
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
