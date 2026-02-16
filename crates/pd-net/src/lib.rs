//! Networking contracts: URL parsing, HTTP messages, and TLS policy.

pub mod client;
pub mod dns;
pub mod http;
pub mod pool;
pub mod tls;
pub mod tls_backend;
pub mod transport;
pub mod url;

use client::Http11Client;
use http::HttpMethod;
use http::HttpRequest;
use pd_core::BrowserResult;
use pd_privacy::PrivacyPolicy;
use pd_security::SecurityPolicy;
use pd_storage::StorageManager;
use tls::StrictTlsPolicy;
use tls::TlsHandshakeConfig;
use url::BrowserUrl;

pub use http::Header;
pub use http::HttpRequestBuilder;
pub use http::HttpResponse;
pub use http::HttpStatusCode;
pub use http::HttpVersion;
pub use pool::ConnectionKey;
pub use tls::TlsVersion;
pub use tls::TrustStoreMode;
pub use url::Scheme;

const DEFAULT_BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";
const DEFAULT_ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const DEFAULT_ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

/// Request prepared by the network layer before transport execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRequest {
    pub request: HttpRequest,
    pub tls: Option<TlsHandshakeConfig>,
}

/// Runtime network stack.
#[derive(Debug, Clone)]
pub struct NetStack {
    pub privacy: PrivacyPolicy,
    pub security: SecurityPolicy,
    pub storage: StorageManager,
    pub tls_policy: StrictTlsPolicy,
}

impl NetStack {
    pub fn new(privacy: PrivacyPolicy, security: SecurityPolicy, storage: StorageManager) -> Self {
        let tls_policy = StrictTlsPolicy::for_security_mode(security.enforce_strict_tls);

        Self {
            privacy,
            security,
            storage,
            tls_policy,
        }
    }

    pub fn parse_url(&self, raw_url: &str) -> BrowserResult<BrowserUrl> {
        BrowserUrl::parse(raw_url)
    }

    pub fn prepare_get(&self, raw_url: &str) -> BrowserResult<PreparedRequest> {
        self.prepare_request(HttpMethod::Get, raw_url)
    }

    pub fn prepare_get_with_tls_policy(
        &self,
        raw_url: &str,
        tls_policy: &StrictTlsPolicy,
    ) -> BrowserResult<PreparedRequest> {
        self.prepare_request_with_tls_policy(HttpMethod::Get, raw_url, tls_policy)
    }

    pub fn prepare_request(
        &self,
        method: HttpMethod,
        raw_url: &str,
    ) -> BrowserResult<PreparedRequest> {
        self.prepare_request_with_tls_policy(method, raw_url, &self.tls_policy)
    }

    pub fn prepare_request_with_tls_policy(
        &self,
        method: HttpMethod,
        raw_url: &str,
        tls_policy: &StrictTlsPolicy,
    ) -> BrowserResult<PreparedRequest> {
        let url = BrowserUrl::parse(raw_url)?;
        let tls = tls_policy.handshake_config_for(&url)?;

        let mut request = HttpRequest::builder(method, url);
        request = request.header("User-Agent", DEFAULT_BROWSER_USER_AGENT)?;
        request = request.header("Accept", DEFAULT_ACCEPT_HEADER)?;
        request = request.header("Accept-Language", DEFAULT_ACCEPT_LANGUAGE)?;
        request = request.header("Accept-Encoding", "gzip, deflate, br")?;
        request = request.header("Upgrade-Insecure-Requests", "1")?;
        request = request.header("Sec-Fetch-Site", "none")?;
        request = request.header("Sec-Fetch-Mode", "navigate")?;
        request = request.header("Sec-Fetch-User", "?1")?;
        request = request.header("Sec-Fetch-Dest", "document")?;

        if self.privacy.block_known_trackers {
            request = request.header("DNT", "1")?;
        }

        Ok(PreparedRequest {
            request: request.build()?,
            tls,
        })
    }

    pub fn http11_client(&self) -> BrowserResult<Http11Client> {
        Http11Client::new(self.tls_policy.clone())
    }

    pub fn http11_client_with_tls_policy(
        &self,
        tls_policy: StrictTlsPolicy,
    ) -> BrowserResult<Http11Client> {
        Http11Client::new(tls_policy)
    }

    pub fn http11_client_webpki_only(&self) -> BrowserResult<Http11Client> {
        let policy = self
            .tls_policy
            .clone()
            .with_trust_store_mode(TrustStoreMode::WebPkiOnly);
        self.http11_client_with_tls_policy(policy)
    }

    pub fn http11_client_with_os_trust_store(&self) -> BrowserResult<Http11Client> {
        let policy = self
            .tls_policy
            .clone()
            .with_trust_store_mode(TrustStoreMode::WebPkiAndOs);
        self.http11_client_with_tls_policy(policy)
    }
}

#[cfg(test)]
mod tests {
    use super::HttpMethod;
    use super::NetStack;
    use pd_privacy::PrivacyPolicy;
    use pd_security::SecurityPolicy;
    use pd_storage::StorageConfig;
    use pd_storage::StorageManager;

    #[test]
    fn strict_mode_blocks_http_urls() {
        let privacy = PrivacyPolicy::default();
        let security = SecurityPolicy::default();
        let storage =
            StorageManager::new(StorageConfig::default(), privacy.clone(), security.clone());
        let stack = NetStack::new(privacy, security, storage);

        let prepared = stack.prepare_request(HttpMethod::Get, "http://example.com/");
        assert!(prepared.is_err());
    }

    #[test]
    fn https_request_prepares_tls_config() {
        let privacy = PrivacyPolicy::default();
        let security = SecurityPolicy::default();
        let storage =
            StorageManager::new(StorageConfig::default(), privacy.clone(), security.clone());
        let stack = NetStack::new(privacy, security, storage);

        let prepared = stack.prepare_request(HttpMethod::Get, "https://example.com/");
        assert!(prepared.is_ok());

        let prepared = match prepared {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };
        assert!(prepared.tls.is_some());
    }
}
