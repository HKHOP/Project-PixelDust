//! URL parsing and validation contracts.

use pd_core::BrowserError;
use pd_core::BrowserResult;
use url::Url;

/// Supported application-level URL schemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    pub fn is_secure(self) -> bool {
        matches!(self, Self::Https)
    }
}

/// Canonical URL object used by the network stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserUrl {
    parsed: Url,
    scheme: Scheme,
    host: String,
    port: u16,
}

impl BrowserUrl {
    pub fn parse(input: &str) -> BrowserResult<Self> {
        let mut parsed = Url::parse(input).map_err(|error| {
            BrowserError::new(
                "net.url.invalid",
                format!("failed to parse URL `{input}`: {error}"),
            )
        })?;

        if parsed.cannot_be_a_base() {
            return Err(BrowserError::new(
                "net.url.invalid_base",
                "URL cannot be used for network navigation",
            ));
        }

        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(BrowserError::new(
                "net.url.credentials_disallowed",
                "URL userinfo (`username:password@`) is not allowed",
            ));
        }

        let scheme = match parsed.scheme() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => {
                return Err(BrowserError::new(
                    "net.url.scheme_unsupported",
                    format!("unsupported scheme `{other}`"),
                ));
            }
        };

        let host = parsed
            .host_str()
            .ok_or_else(|| BrowserError::new("net.url.host_missing", "URL must include a host"))?
            .to_ascii_lowercase();

        let port = parsed.port_or_known_default().ok_or_else(|| {
            BrowserError::new(
                "net.url.port_missing",
                "unable to determine effective port for URL",
            )
        })?;

        // Fragments are client-side only and never sent on the wire.
        parsed.set_fragment(None);

        Ok(Self {
            parsed,
            scheme,
            host,
            port,
        })
    }

    pub fn as_str(&self) -> &str {
        self.parsed.as_str()
    }

    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_secure(&self) -> bool {
        self.scheme.is_secure()
    }

    pub fn authority(&self) -> String {
        if self.port == default_port(self.scheme) {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    pub fn origin(&self) -> String {
        format!("{}://{}", self.scheme.as_str(), self.authority())
    }

    pub fn path_and_query(&self) -> String {
        let path = if self.parsed.path().is_empty() {
            "/"
        } else {
            self.parsed.path()
        };

        match self.parsed.query() {
            Some(query) => format!("{path}?{query}"),
            None => path.to_owned(),
        }
    }
}

fn default_port(scheme: Scheme) -> u16 {
    match scheme {
        Scheme::Http => 80,
        Scheme::Https => 443,
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserUrl;

    #[test]
    fn parses_https_url() {
        let parsed = BrowserUrl::parse("https://example.com/path?q=1");
        assert!(parsed.is_ok());

        let parsed = match parsed {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(parsed.host(), "example.com");
        assert_eq!(parsed.port(), 443);
        assert_eq!(parsed.path_and_query(), "/path?q=1");
        assert!(parsed.is_secure());
    }

    #[test]
    fn removes_fragment_from_canonical_url() {
        let parsed = BrowserUrl::parse("https://example.com/path#section");
        assert!(parsed.is_ok());

        let parsed = match parsed {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };
        assert_eq!(parsed.as_str(), "https://example.com/path");
    }

    #[test]
    fn rejects_unsupported_scheme() {
        let parsed = BrowserUrl::parse("ftp://example.com/file.txt");
        assert!(parsed.is_err());
    }

    #[test]
    fn rejects_embedded_credentials() {
        let parsed = BrowserUrl::parse("https://user:pass@example.com/");
        assert!(parsed.is_err());
    }
}
