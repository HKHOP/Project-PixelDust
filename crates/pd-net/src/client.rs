//! HTTP/1.1 client built on DNS, transport, TLS, and pooling contracts.

use crate::PreparedRequest;
use crate::dns::DnsResolver;
use crate::dns::SystemDnsResolver;
use crate::http::Header;
use crate::http::HttpRequest;
use crate::http::HttpResponse;
use crate::http::HttpStatusCode;
use crate::http::HttpVersion;
use crate::pool::ConnectionKey;
use crate::pool::ConnectionPool;
use crate::pool::InMemoryConnectionPool;
use crate::pool::PoolStats;
use crate::tls::StrictTlsPolicy;
use crate::tls_backend::RustlsTlsAdapter;
use crate::tls_backend::TlsBackendAdapter;
use crate::transport::BoxedIoStream;
use crate::transport::TcpTransport;
use crate::transport::Transport;
use brotli::Decompressor;
use flate2::read::DeflateDecoder;
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

const MAX_RESPONSE_HEAD_BYTES: usize = 128 * 1024;
const MAX_CHUNK_LINE_BYTES: usize = 8 * 1024;

/// HTTP/1.1 client with pluggable resolver/transport/pool/tls backend.
pub struct Http11Client<
    R = SystemDnsResolver,
    T = TcpTransport,
    P = InMemoryConnectionPool,
    A = RustlsTlsAdapter,
> where
    R: DnsResolver,
    T: Transport,
    P: ConnectionPool,
    A: TlsBackendAdapter,
{
    dns: R,
    transport: T,
    pool: P,
    tls_adapter: A,
    tls_policy: StrictTlsPolicy,
    connect_timeout: Duration,
}

impl Http11Client {
    pub fn new(tls_policy: StrictTlsPolicy) -> BrowserResult<Self> {
        Self::with_parts(
            SystemDnsResolver,
            TcpTransport,
            InMemoryConnectionPool::default(),
            RustlsTlsAdapter,
            tls_policy,
        )
    }
}

impl<R, T, P, A> Http11Client<R, T, P, A>
where
    R: DnsResolver,
    T: Transport,
    P: ConnectionPool,
    A: TlsBackendAdapter,
{
    pub fn with_parts(
        dns: R,
        transport: T,
        pool: P,
        tls_adapter: A,
        tls_policy: StrictTlsPolicy,
    ) -> BrowserResult<Self> {
        tls_policy.validate()?;
        Ok(Self {
            dns,
            transport,
            pool,
            tls_adapter,
            tls_policy,
            connect_timeout: Duration::from_secs(10),
        })
    }

    pub fn set_connect_timeout(&mut self, timeout: Duration) {
        self.connect_timeout = timeout;
    }

    pub fn pool_stats(&self) -> PoolStats {
        self.pool.stats()
    }

    pub fn execute(&mut self, prepared: PreparedRequest) -> BrowserResult<HttpResponse> {
        self.tls_policy.validate()?;
        validate_prepared_request(&prepared)?;

        let key = ConnectionKey::from_url(&prepared.request.url);
        let mut stream = match self.pool.checkout(&key) {
            Some(existing) => existing,
            None => self.open_stream(&prepared)?,
        };

        write_request(&mut *stream, &prepared.request)?;
        let outcome = read_response(&mut *stream, &prepared.request)?;

        if outcome.reusable {
            self.pool.checkin(key, stream);
        }

        Ok(outcome.response)
    }

    fn open_stream(&self, prepared: &PreparedRequest) -> BrowserResult<BoxedIoStream> {
        let host = prepared.request.url.host();
        let port = prepared.request.url.port();
        let addresses = self.dns.resolve(host, port)?;

        connect_first_available(&self.transport, &addresses, self.connect_timeout).and_then(
            |stream| match &prepared.tls {
                Some(handshake) => {
                    self.tls_adapter
                        .connect_tls(stream, handshake, &self.tls_policy)
                }
                None => Ok(Box::new(stream)),
            },
        )
    }
}

fn validate_prepared_request(prepared: &PreparedRequest) -> BrowserResult<()> {
    if prepared.request.url.is_secure() && prepared.tls.is_none() {
        return Err(BrowserError::new(
            "net.http.tls_missing",
            "HTTPS request is missing TLS handshake configuration",
        ));
    }

    if !prepared.request.url.is_secure() && prepared.tls.is_some() {
        return Err(BrowserError::new(
            "net.http.tls_unexpected",
            "non-HTTPS request must not include TLS handshake configuration",
        ));
    }

    Ok(())
}

fn connect_first_available<T: Transport>(
    transport: &T,
    addresses: &[SocketAddr],
    timeout: Duration,
) -> BrowserResult<TcpStream> {
    let mut last_error: Option<BrowserError> = None;

    for address in addresses {
        match transport.connect(*address, timeout) {
            Ok(stream) => return Ok(stream),
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    match last_error {
        Some(error) => Err(error),
        None => Err(BrowserError::new(
            "net.transport.no_addresses",
            "no addresses available to open a connection",
        )),
    }
}

fn write_request(stream: &mut dyn Write, request: &HttpRequest) -> BrowserResult<()> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(request.method.as_str().as_bytes());
    encoded.push(b' ');
    encoded.extend_from_slice(request.request_target().as_bytes());
    encoded.push(b' ');
    encoded.extend_from_slice(request.version.as_str().as_bytes());
    encoded.extend_from_slice(b"\r\n");

    for header in &request.headers {
        encoded.extend_from_slice(header.name.as_bytes());
        encoded.extend_from_slice(b": ");
        encoded.extend_from_slice(header.value.as_bytes());
        encoded.extend_from_slice(b"\r\n");
    }
    encoded.extend_from_slice(b"\r\n");
    encoded.extend_from_slice(&request.body);

    stream.write_all(&encoded).map_err(|error| {
        BrowserError::new(
            "net.http.write_failed",
            format!("failed to write HTTP request bytes: {error}"),
        )
    })?;
    stream.flush().map_err(|error| {
        BrowserError::new(
            "net.http.flush_failed",
            format!("failed to flush HTTP request bytes: {error}"),
        )
    })?;

    Ok(())
}

struct ResponseReadOutcome {
    response: HttpResponse,
    reusable: bool,
}

fn read_response(
    stream: &mut dyn Read,
    request: &HttpRequest,
) -> BrowserResult<ResponseReadOutcome> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end: Option<usize> = None;

    while header_end.is_none() {
        let read = stream.read(&mut chunk).map_err(|error| {
            BrowserError::new(
                "net.http.read_head_failed",
                format!("failed while reading HTTP response head: {error}"),
            )
        })?;

        if read == 0 {
            return Err(BrowserError::new(
                "net.http.unexpected_eof",
                "unexpected EOF before response head completed",
            ));
        }

        buffer.extend_from_slice(&chunk[..read]);
        if buffer.len() > MAX_RESPONSE_HEAD_BYTES {
            return Err(BrowserError::new(
                "net.http.head_too_large",
                format!("HTTP response head exceeds {MAX_RESPONSE_HEAD_BYTES} bytes"),
            ));
        }

        header_end = find_header_end(&buffer);
    }

    let header_end = match header_end {
        Some(value) => value,
        None => {
            return Err(BrowserError::new(
                "net.http.header_terminator_missing",
                "response head terminator not found",
            ));
        }
    };

    let head_bytes = &buffer[..header_end];
    let mut body_bytes = buffer[header_end..].to_vec();
    let head_text = std::str::from_utf8(head_bytes).map_err(|error| {
        BrowserError::new(
            "net.http.head_invalid_utf8",
            format!("HTTP response head is not valid UTF-8 text: {error}"),
        )
    })?;

    let mut lines = head_text.split("\r\n");
    let status_line = lines.next().ok_or_else(|| {
        BrowserError::new("net.http.status_line_missing", "missing HTTP status line")
    })?;
    let (version, status) = parse_status_line(status_line)?;

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }

        let (name, value) = line.split_once(':').ok_or_else(|| {
            BrowserError::new(
                "net.http.header_invalid",
                format!("invalid HTTP header line `{line}`"),
            )
        })?;
        let header = Header::new(name.trim(), value.trim())?;
        headers.push(header);
    }

    let has_transfer_encoding = headers
        .iter()
        .any(|header| header.name.eq_ignore_ascii_case("transfer-encoding"));
    let has_chunked_transfer = header_contains(&headers, "transfer-encoding", "chunked");
    if has_transfer_encoding && !has_chunked_transfer {
        return Err(BrowserError::new(
            "net.http.transfer_encoding_unsupported",
            "only chunked transfer encoding is currently supported",
        ));
    }

    let content_length = if has_chunked_transfer {
        None
    } else {
        parse_content_length(&headers)?
    };
    let has_no_body = request.method.as_str() == "HEAD" || status_disallows_body(status.as_u16());

    let reusable = if has_no_body {
        true
    } else if has_chunked_transfer {
        body_bytes = read_chunked_body(stream, body_bytes)?;
        true
    } else if let Some(len) = content_length {
        if body_bytes.len() < len {
            let remaining = len - body_bytes.len();
            let mut rest = vec![0_u8; remaining];
            stream.read_exact(&mut rest).map_err(|error| {
                BrowserError::new(
                    "net.http.read_body_failed",
                    format!("failed to read HTTP body bytes: {error}"),
                )
            })?;
            body_bytes.extend_from_slice(&rest);
        } else if body_bytes.len() > len {
            body_bytes.truncate(len);
        }

        true
    } else if header_contains(&headers, "connection", "close") {
        let mut tail = Vec::new();
        stream.read_to_end(&mut tail).map_err(|error| {
            BrowserError::new(
                "net.http.read_body_failed",
                format!("failed while draining connection-close response body: {error}"),
            )
        })?;
        body_bytes.extend_from_slice(&tail);
        false
    } else {
        return Err(BrowserError::new(
            "net.http.body_length_unknown",
            "response body length is unknown without Content-Length or Connection: close",
        ));
    };

    if !has_no_body {
        body_bytes = decode_content_encoding(&headers, &body_bytes)?;
    }

    let response = HttpResponse {
        version,
        status,
        headers,
        body: if has_no_body { Vec::new() } else { body_bytes },
    };

    Ok(ResponseReadOutcome {
        reusable: reusable && is_keep_alive(request, &response),
        response,
    })
}

struct PrefixedStreamReader<'a> {
    prefetched: Vec<u8>,
    offset: usize,
    stream: &'a mut dyn Read,
}

impl<'a> PrefixedStreamReader<'a> {
    fn new(stream: &'a mut dyn Read, prefetched: Vec<u8>) -> Self {
        Self {
            prefetched,
            offset: 0,
            stream,
        }
    }

    fn read_exact_into(
        &mut self,
        out: &mut [u8],
        code: &'static str,
        detail: &str,
    ) -> BrowserResult<()> {
        let available = self.prefetched.len().saturating_sub(self.offset);
        let prefix_take = available.min(out.len());

        if prefix_take > 0 {
            out[..prefix_take]
                .copy_from_slice(&self.prefetched[self.offset..self.offset + prefix_take]);
            self.offset += prefix_take;
        }

        if prefix_take < out.len() {
            self.stream
                .read_exact(&mut out[prefix_take..])
                .map_err(|error| BrowserError::new(code, format!("{detail}: {error}")))?;
        }

        Ok(())
    }
}

fn read_chunked_body(stream: &mut dyn Read, prefetched: Vec<u8>) -> BrowserResult<Vec<u8>> {
    let mut reader = PrefixedStreamReader::new(stream, prefetched);
    let mut decoded = Vec::new();

    loop {
        let size_line = read_crlf_line(&mut reader)?;
        if size_line.is_empty() {
            continue;
        }

        let size_token = size_line.split(';').next().unwrap_or_default().trim();
        let chunk_size = usize::from_str_radix(size_token, 16).map_err(|error| {
            BrowserError::new(
                "net.http.chunk_size_invalid",
                format!("invalid chunk size `{size_token}`: {error}"),
            )
        })?;

        if chunk_size == 0 {
            drain_chunk_trailers(&mut reader)?;
            break;
        }

        let start = decoded.len();
        decoded.resize(start + chunk_size, 0);
        reader.read_exact_into(
            &mut decoded[start..],
            "net.http.read_body_failed",
            "failed while reading chunked HTTP body bytes",
        )?;

        let mut terminator = [0_u8; 2];
        reader.read_exact_into(
            &mut terminator,
            "net.http.read_body_failed",
            "failed while reading chunked body terminator",
        )?;
        if terminator != *b"\r\n" {
            return Err(BrowserError::new(
                "net.http.chunk_terminator_invalid",
                "chunk data is missing trailing CRLF",
            ));
        }
    }

    Ok(decoded)
}

fn drain_chunk_trailers(reader: &mut PrefixedStreamReader<'_>) -> BrowserResult<()> {
    loop {
        let line = read_crlf_line(reader)?;
        if line.is_empty() {
            break;
        }

        if line.split_once(':').is_none() {
            return Err(BrowserError::new(
                "net.http.chunk_trailer_invalid",
                format!("invalid chunk trailer line `{line}`"),
            ));
        }
    }

    Ok(())
}

fn read_crlf_line(reader: &mut PrefixedStreamReader<'_>) -> BrowserResult<String> {
    let mut line = Vec::new();

    loop {
        let mut byte = [0_u8; 1];
        reader.read_exact_into(
            &mut byte,
            "net.http.read_body_failed",
            "failed while reading chunked transfer line",
        )?;
        line.push(byte[0]);

        if line.len() > MAX_CHUNK_LINE_BYTES {
            return Err(BrowserError::new(
                "net.http.chunk_line_too_large",
                format!("chunk metadata line exceeds {MAX_CHUNK_LINE_BYTES} bytes"),
            ));
        }

        if line.len() >= 2 && line[line.len() - 2..] == *b"\r\n" {
            line.truncate(line.len() - 2);
            return String::from_utf8(line).map_err(|error| {
                BrowserError::new(
                    "net.http.chunk_line_invalid_utf8",
                    format!("chunk metadata line is not valid UTF-8: {error}"),
                )
            });
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|idx| idx + 4)
}

fn parse_status_line(line: &str) -> BrowserResult<(HttpVersion, HttpStatusCode)> {
    let mut parts = line.splitn(3, ' ');
    let version = parts.next().ok_or_else(|| {
        BrowserError::new(
            "net.http.status_line_invalid",
            format!("missing HTTP version in status line `{line}`"),
        )
    })?;

    let code_text = parts.next().ok_or_else(|| {
        BrowserError::new(
            "net.http.status_line_invalid",
            format!("missing status code in status line `{line}`"),
        )
    })?;

    let version = match version {
        "HTTP/1.0" => HttpVersion::Http10,
        "HTTP/1.1" => HttpVersion::Http11,
        "HTTP/2" => HttpVersion::Http2,
        other => {
            return Err(BrowserError::new(
                "net.http.version_unsupported",
                format!("unsupported response version `{other}`"),
            ));
        }
    };

    let code_value = code_text.parse::<u16>().map_err(|error| {
        BrowserError::new(
            "net.http.status_line_invalid",
            format!("invalid status code `{code_text}`: {error}"),
        )
    })?;

    let code = HttpStatusCode::new(code_value)?;
    Ok((version, code))
}

fn parse_content_length(headers: &[Header]) -> BrowserResult<Option<usize>> {
    let mut value: Option<usize> = None;
    for header in headers {
        if header.name.eq_ignore_ascii_case("content-length") {
            let parsed = header.value.trim().parse::<usize>().map_err(|error| {
                BrowserError::new(
                    "net.http.content_length_invalid",
                    format!("invalid Content-Length `{}`: {error}", header.value),
                )
            })?;

            if let Some(existing) = value {
                if existing != parsed {
                    return Err(BrowserError::new(
                        "net.http.content_length_conflict",
                        "conflicting Content-Length headers in response",
                    ));
                }
            } else {
                value = Some(parsed);
            }
        }
    }

    Ok(value)
}

fn status_disallows_body(status_code: u16) -> bool {
    (100..200).contains(&status_code) || status_code == 204 || status_code == 304
}

fn is_keep_alive(request: &HttpRequest, response: &HttpResponse) -> bool {
    if request
        .header("Connection")
        .is_some_and(|value| value.eq_ignore_ascii_case("close"))
    {
        return false;
    }

    if header_contains(&response.headers, "connection", "close") {
        return false;
    }

    match response.version {
        HttpVersion::Http10 => header_contains(&response.headers, "connection", "keep-alive"),
        HttpVersion::Http11 => true,
        HttpVersion::Http2 => true,
    }
}

fn header_contains(headers: &[Header], name: &str, value: &str) -> bool {
    headers.iter().any(|header| {
        header.name.eq_ignore_ascii_case(name)
            && header
                .value
                .split(',')
                .any(|token| token.trim().eq_ignore_ascii_case(value))
    })
}

fn decode_content_encoding(headers: &[Header], body: &[u8]) -> BrowserResult<Vec<u8>> {
    let encodings = content_encodings(headers);
    if encodings.is_empty() {
        return Ok(body.to_vec());
    }

    let mut decoded = body.to_vec();
    for encoding in encodings.iter().rev() {
        decoded = match encoding.as_str() {
            "identity" => decoded,
            "gzip" | "x-gzip" => decode_gzip(&decoded)?,
            "deflate" => decode_deflate(&decoded)?,
            "br" => decode_brotli(&decoded)?,
            _ => {
                return Err(BrowserError::new(
                    "net.http.content_encoding_unsupported",
                    format!("unsupported content encoding `{encoding}`"),
                ));
            }
        };
    }

    Ok(decoded)
}

fn content_encodings(headers: &[Header]) -> Vec<String> {
    let mut encodings = Vec::new();
    for header in headers {
        if !header.name.eq_ignore_ascii_case("content-encoding") {
            continue;
        }

        for token in header.value.split(',') {
            let value = token.trim().to_ascii_lowercase();
            if !value.is_empty() {
                encodings.push(value);
            }
        }
    }

    encodings
}

fn decode_gzip(body: &[u8]) -> BrowserResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(Cursor::new(body));
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).map_err(|error| {
        BrowserError::new(
            "net.http.decode_failed",
            format!("gzip decode failed: {error}"),
        )
    })?;
    Ok(decoded)
}

fn decode_deflate(body: &[u8]) -> BrowserResult<Vec<u8>> {
    let mut zlib_decoder = ZlibDecoder::new(Cursor::new(body));
    let mut zlib_decoded = Vec::new();
    if zlib_decoder.read_to_end(&mut zlib_decoded).is_ok() {
        return Ok(zlib_decoded);
    }

    let mut raw_decoder = DeflateDecoder::new(Cursor::new(body));
    let mut raw_decoded = Vec::new();
    raw_decoder.read_to_end(&mut raw_decoded).map_err(|error| {
        BrowserError::new(
            "net.http.decode_failed",
            format!("deflate decode failed: {error}"),
        )
    })?;
    Ok(raw_decoded)
}

fn decode_brotli(body: &[u8]) -> BrowserResult<Vec<u8>> {
    let mut decoder = Decompressor::new(Cursor::new(body), 4096);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).map_err(|error| {
        BrowserError::new(
            "net.http.decode_failed",
            format!("brotli decode failed: {error}"),
        )
    })?;
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::decode_content_encoding;
    use super::find_header_end;
    use super::parse_status_line;
    use super::read_chunked_body;
    use super::read_response;
    use super::status_disallows_body;
    use crate::http::Header;
    use crate::http::HttpMethod;
    use crate::http::HttpRequest;
    use crate::url::BrowserUrl;
    use brotli::CompressorWriter;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use flate2::write::ZlibEncoder;
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn header_terminator_is_detected() {
        let data = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        let end = find_header_end(data);
        assert_eq!(end, Some(data.len()));
    }

    #[test]
    fn status_line_parser_handles_http_11() {
        let parsed = parse_status_line("HTTP/1.1 200 OK");
        assert!(parsed.is_ok());
    }

    #[test]
    fn status_line_parser_handles_http_10() {
        let parsed = parse_status_line("HTTP/1.0 200 OK");
        assert!(parsed.is_ok());
    }

    #[test]
    fn detects_bodyless_status_codes() {
        assert!(status_disallows_body(101));
        assert!(status_disallows_body(204));
        assert!(status_disallows_body(304));
        assert!(!status_disallows_body(200));
    }

    #[test]
    fn decodes_chunked_body() {
        let prefetched = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n".to_vec();
        let mut stream = Cursor::new(Vec::<u8>::new());
        let decoded = read_chunked_body(&mut stream, prefetched);
        assert_eq!(decoded, Ok(b"Wikipedia".to_vec()));
    }

    #[test]
    fn chunked_decode_reports_invalid_size() {
        let prefetched = b"Z\r\nx\r\n0\r\n\r\n".to_vec();
        let mut stream = Cursor::new(Vec::<u8>::new());
        let decoded = read_chunked_body(&mut stream, prefetched);
        assert!(decoded.is_err());
        if let Err(error) = decoded {
            assert_eq!(error.code, "net.http.chunk_size_invalid");
        }
    }

    #[test]
    fn read_response_handles_chunked_transfer_encoding() {
        let url = BrowserUrl::parse("https://example.com/chunked");
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

        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\
                    4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        let mut stream = Cursor::new(raw.to_vec());
        let outcome = read_response(&mut stream, &request);
        assert!(outcome.is_ok());
        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(outcome.response.body, b"Wikipedia");
        assert!(outcome.reusable);
    }

    #[test]
    fn http10_response_is_not_reused_without_keep_alive() {
        let url = BrowserUrl::parse("http://localhost:3000/");
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

        let raw = b"HTTP/1.0 200 OK\r\nContent-Length: 2\r\n\r\nok";
        let mut stream = Cursor::new(raw.to_vec());
        let outcome = read_response(&mut stream, &request);
        assert!(outcome.is_ok());
        let outcome = match outcome {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        assert_eq!(outcome.response.status.as_u16(), 200);
        assert_eq!(outcome.response.body, b"ok");
        assert!(!outcome.reusable);
    }

    #[test]
    fn rejects_unsupported_transfer_encoding() {
        let url = BrowserUrl::parse("https://example.com/unsupported-te");
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

        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nConnection: close\r\n\r\nbody";
        let mut stream = Cursor::new(raw.to_vec());
        let outcome = read_response(&mut stream, &request);
        assert!(outcome.is_err());
        if let Err(error) = outcome {
            assert_eq!(error.code, "net.http.transfer_encoding_unsupported");
        }
    }

    #[test]
    fn decodes_gzip_content_encoding() {
        let mut encoded = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut encoded, Compression::default());
            let wrote = encoder.write_all(b"hello gzip");
            assert!(wrote.is_ok());
            let finish = encoder.finish();
            assert!(finish.is_ok());
        }

        let header = Header::new("Content-Encoding", "gzip");
        assert!(header.is_ok());
        let header = match header {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let decoded = decode_content_encoding(&[header], &encoded);
        assert_eq!(decoded, Ok(b"hello gzip".to_vec()));
    }

    #[test]
    fn decodes_deflate_content_encoding() {
        let mut encoded = Vec::new();
        {
            let mut encoder = ZlibEncoder::new(&mut encoded, Compression::default());
            let wrote = encoder.write_all(b"hello deflate");
            assert!(wrote.is_ok());
            let finish = encoder.finish();
            assert!(finish.is_ok());
        }

        let header = Header::new("Content-Encoding", "deflate");
        assert!(header.is_ok());
        let header = match header {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let decoded = decode_content_encoding(&[header], &encoded);
        assert_eq!(decoded, Ok(b"hello deflate".to_vec()));
    }

    #[test]
    fn decodes_brotli_content_encoding() {
        let mut encoded = Vec::new();
        {
            let mut writer = CompressorWriter::new(&mut encoded, 4096, 5, 22);
            let wrote = writer.write_all(b"hello br");
            assert!(wrote.is_ok());
            let flushed = writer.flush();
            assert!(flushed.is_ok());
        }

        let header = Header::new("Content-Encoding", "br");
        assert!(header.is_ok());
        let header = match header {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        };

        let decoded = decode_content_encoding(&[header], &encoded);
        assert_eq!(decoded, Ok(b"hello br".to_vec()));
    }
}
