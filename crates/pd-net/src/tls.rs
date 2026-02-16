//! Strict TLS policy contracts.

use crate::url::BrowserUrl;
use crate::url::Scheme;
use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::net::IpAddr;

/// Supported TLS protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TlsVersion {
    V1_2,
    V1_3,
}

impl TlsVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V1_2 => "TLS1.2",
            Self::V1_3 => "TLS1.3",
        }
    }
}

/// Controls which trust anchors are used for server certificate verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustStoreMode {
    /// Use only the embedded Mozilla/WebPKI roots.
    WebPkiOnly,
    /// Use WebPKI roots and merge operating-system roots (enterprise/local CAs).
    WebPkiAndOs,
}

/// TLS handshake requirements for HTTPS requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsHandshakeConfig {
    pub server_name: String,
    pub minimum_version: TlsVersion,
    pub maximum_version: TlsVersion,
    pub alpn_protocols: Vec<String>,
    pub require_sni: bool,
    pub require_ocsp_stapling: bool,
}

/// Security policy that defines strict TLS behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictTlsPolicy {
    pub minimum_version: TlsVersion,
    pub maximum_version: TlsVersion,
    pub require_sni: bool,
    pub require_ocsp_stapling: bool,
    pub allow_invalid_certificates: bool,
    pub allow_legacy_cipher_suites: bool,
    pub https_only_mode: bool,
    pub trust_store_mode: TrustStoreMode,
}

impl Default for StrictTlsPolicy {
    fn default() -> Self {
        Self {
            minimum_version: TlsVersion::V1_2,
            maximum_version: TlsVersion::V1_3,
            require_sni: true,
            require_ocsp_stapling: true,
            allow_invalid_certificates: false,
            allow_legacy_cipher_suites: false,
            https_only_mode: false,
            trust_store_mode: TrustStoreMode::WebPkiOnly,
        }
    }
}

impl StrictTlsPolicy {
    pub fn for_security_mode(enforce_strict_tls: bool) -> Self {
        Self {
            https_only_mode: enforce_strict_tls,
            ..Self::default()
        }
    }

    pub fn with_trust_store_mode(mut self, mode: TrustStoreMode) -> Self {
        self.trust_store_mode = mode;
        self
    }

    pub fn with_ocsp_stapling_required(mut self, required: bool) -> Self {
        self.require_ocsp_stapling = required;
        self
    }

    pub fn validate(&self) -> BrowserResult<()> {
        if self.minimum_version > self.maximum_version {
            return Err(BrowserError::new(
                "net.tls.invalid_version_range",
                "minimum TLS version cannot be greater than maximum version",
            ));
        }

        if self.allow_invalid_certificates {
            return Err(BrowserError::new(
                "net.tls.invalid_certificate_mode",
                "strict TLS policy forbids invalid certificates",
            ));
        }

        if self.allow_legacy_cipher_suites {
            return Err(BrowserError::new(
                "net.tls.legacy_cipher_mode",
                "strict TLS policy forbids legacy cipher suites",
            ));
        }

        Ok(())
    }

    pub fn handshake_config_for(
        &self,
        url: &BrowserUrl,
    ) -> BrowserResult<Option<TlsHandshakeConfig>> {
        self.validate()?;

        match url.scheme() {
            Scheme::Http => {
                if self.https_only_mode {
                    return Err(BrowserError::new(
                        "net.tls.https_only",
                        "HTTPS-only mode blocks plain HTTP navigation",
                    ));
                }

                Ok(None)
            }
            Scheme::Https => {
                if self.require_sni && is_ip_address(url.host()) {
                    return Err(BrowserError::new(
                        "net.tls.sni_host_invalid",
                        "SNI requires a DNS host, not a raw IP address",
                    ));
                }

                Ok(Some(TlsHandshakeConfig {
                    server_name: url.host().to_owned(),
                    minimum_version: self.minimum_version,
                    maximum_version: self.maximum_version,
                    alpn_protocols: vec!["http/1.1".to_owned()],
                    require_sni: self.require_sni,
                    require_ocsp_stapling: self.require_ocsp_stapling,
                }))
            }
        }
    }
}

fn is_ip_address(host: &str) -> bool {
    host.parse::<IpAddr>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::StrictTlsPolicy;
    use super::TlsVersion;
    use super::TrustStoreMode;
    use crate::url::BrowserUrl;

    #[test]
    fn validates_version_range() {
        let policy = StrictTlsPolicy {
            minimum_version: TlsVersion::V1_3,
            maximum_version: TlsVersion::V1_2,
            ..StrictTlsPolicy::default()
        };

        assert!(policy.validate().is_err());
    }

    #[test]
    fn creates_handshake_for_https() {
        let policy = StrictTlsPolicy::default();
        let url = BrowserUrl::parse("https://example.com/");
        assert!(url.is_ok());

        let url = match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let handshake = policy.handshake_config_for(&url);
        assert!(handshake.is_ok());

        let handshake = match handshake {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };
        assert!(handshake.is_some());
    }

    #[test]
    fn https_only_mode_blocks_http() {
        let policy = StrictTlsPolicy::for_security_mode(true);
        let url = BrowserUrl::parse("http://example.com/");
        assert!(url.is_ok());

        let url = match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let handshake = policy.handshake_config_for(&url);
        assert!(handshake.is_err());
    }

    #[test]
    fn defaults_to_webpki_only() {
        let policy = StrictTlsPolicy::default();
        assert_eq!(policy.trust_store_mode, TrustStoreMode::WebPkiOnly);
    }

    #[test]
    fn trust_store_mode_can_be_overridden() {
        let policy = StrictTlsPolicy::default().with_trust_store_mode(TrustStoreMode::WebPkiAndOs);
        assert_eq!(policy.trust_store_mode, TrustStoreMode::WebPkiAndOs);
    }
}
