//! HTTP request/response contracts.

use crate::url::BrowserUrl;
use pd_core::BrowserError;
use pd_core::BrowserResult;

/// Supported outbound HTTP methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Options,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Options => "OPTIONS",
        }
    }
}

/// HTTP protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
}

impl HttpVersion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http10 => "HTTP/1.0",
            Self::Http11 => "HTTP/1.1",
            Self::Http2 => "HTTP/2",
        }
    }
}

/// Single HTTP header with validated wire-safe name/value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new(name: &str, value: &str) -> BrowserResult<Self> {
        if !is_valid_header_name(name) {
            return Err(BrowserError::new(
                "net.http.header_name_invalid",
                format!("invalid HTTP header name `{name}`"),
            ));
        }

        if value.bytes().any(|byte| matches!(byte, b'\r' | b'\n' | 0)) {
            return Err(BrowserError::new(
                "net.http.header_value_invalid",
                format!("invalid characters found in HTTP header `{name}`"),
            ));
        }

        Ok(Self {
            name: name.to_owned(),
            value: value.to_owned(),
        })
    }
}

/// Outgoing HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: BrowserUrl,
    pub version: HttpVersion,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn builder(method: HttpMethod, url: BrowserUrl) -> HttpRequestBuilder {
        HttpRequestBuilder {
            method,
            url,
            version: HttpVersion::Http11,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn request_target(&self) -> String {
        self.url.path_and_query()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }
}

/// Builder for `HttpRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequestBuilder {
    method: HttpMethod,
    url: BrowserUrl,
    version: HttpVersion,
    headers: Vec<Header>,
    body: Vec<u8>,
}

impl HttpRequestBuilder {
    pub fn version(mut self, version: HttpVersion) -> Self {
        self.version = version;
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> BrowserResult<Self> {
        self.headers.push(Header::new(name, value)?);
        Ok(self)
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn build(mut self) -> BrowserResult<HttpRequest> {
        if matches!(self.method, HttpMethod::Get | HttpMethod::Head) && !self.body.is_empty() {
            return Err(BrowserError::new(
                "net.http.body_disallowed",
                format!("{} requests must not include a body", self.method.as_str()),
            ));
        }

        ensure_singleton_header(&self.headers, "host")?;
        ensure_singleton_header(&self.headers, "content-length")?;

        if !has_header(&self.headers, "host") {
            let host = self.url.authority();
            self.headers.push(Header::new("Host", &host)?);
        }

        if !self.body.is_empty() && !has_header(&self.headers, "content-length") {
            let len = self.body.len().to_string();
            self.headers.push(Header::new("Content-Length", &len)?);
        }

        Ok(HttpRequest {
            method: self.method,
            url: self.url,
            version: self.version,
            headers: self.headers,
            body: self.body,
        })
    }
}

/// HTTP status code wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HttpStatusCode(u16);

impl HttpStatusCode {
    pub fn new(code: u16) -> BrowserResult<Self> {
        if (100..=599).contains(&code) {
            return Ok(Self(code));
        }

        Err(BrowserError::new(
            "net.http.status_invalid",
            format!("status code must be 100-599, got `{code}`"),
        ))
    }

    pub fn as_u16(self) -> u16 {
        self.0
    }

    pub fn is_success(self) -> bool {
        (200..=299).contains(&self.0)
    }
}

/// Incoming HTTP response contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: HttpStatusCode,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
}

fn ensure_singleton_header(headers: &[Header], name: &str) -> BrowserResult<()> {
    let count = headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case(name))
        .count();

    if count <= 1 {
        return Ok(());
    }

    Err(BrowserError::new(
        "net.http.duplicate_header",
        format!("header `{name}` must appear at most once"),
    ))
}

fn has_header(headers: &[Header], name: &str) -> bool {
    headers
        .iter()
        .any(|header| header.name.eq_ignore_ascii_case(name))
}

fn is_valid_header_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    name.bytes().all(is_token_char)
}

fn is_token_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

#[cfg(test)]
mod tests {
    use super::HttpMethod;
    use super::HttpRequest;
    use super::HttpStatusCode;
    use crate::url::BrowserUrl;

    #[test]
    fn host_header_is_added_automatically() {
        let url = BrowserUrl::parse("https://example.com/path");
        assert!(url.is_ok());
        let url = match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let request = HttpRequest::builder(HttpMethod::Get, url).build();
        assert!(request.is_ok());
        let request = match request {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(request.header("Host"), Some("example.com"));
        assert_eq!(request.request_target(), "/path");
    }

    #[test]
    fn get_request_cannot_have_body() {
        let url = BrowserUrl::parse("https://example.com/");
        assert!(url.is_ok());
        let url = match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let request = HttpRequest::builder(HttpMethod::Get, url)
            .body(vec![1, 2, 3])
            .build();
        assert!(request.is_err());
    }

    #[test]
    fn content_length_is_added_when_body_present() {
        let url = BrowserUrl::parse("https://example.com/api");
        assert!(url.is_ok());
        let url = match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let request = HttpRequest::builder(HttpMethod::Post, url)
            .header("Content-Type", "application/json")
            .and_then(|builder| builder.body(b"{}".to_vec()).build());
        assert!(request.is_ok());

        let request = match request {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };
        assert_eq!(request.header("Content-Length"), Some("2"));
    }

    #[test]
    fn status_code_range_is_enforced() {
        assert!(HttpStatusCode::new(200).is_ok());
        assert!(HttpStatusCode::new(99).is_err());
        assert!(HttpStatusCode::new(600).is_err());
    }
}
