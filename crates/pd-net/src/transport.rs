//! Transport contracts and default TCP implementation.

use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

/// Trait-object-safe stream used by HTTP client and pool contracts.
pub trait IoStream: Read + Write {}
impl<T> IoStream for T where T: Read + Write {}

pub type BoxedIoStream = Box<dyn IoStream>;

/// Low-level transport abstraction for opening TCP connections.
pub trait Transport {
    fn connect(&self, address: SocketAddr, timeout: Duration) -> BrowserResult<TcpStream>;
}

/// Standard library TCP transport.
#[derive(Debug, Clone, Copy, Default)]
pub struct TcpTransport;

impl Transport for TcpTransport {
    fn connect(&self, address: SocketAddr, timeout: Duration) -> BrowserResult<TcpStream> {
        let stream = TcpStream::connect_timeout(&address, timeout).map_err(|error| {
            BrowserError::new(
                "net.transport.connect_failed",
                format!("failed to connect to `{address}`: {error}"),
            )
        })?;

        stream.set_nodelay(true).map_err(|error| {
            BrowserError::new(
                "net.transport.nodelay_failed",
                format!("failed to enable TCP_NODELAY for `{address}`: {error}"),
            )
        })?;

        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            BrowserError::new(
                "net.transport.read_timeout_failed",
                format!("failed to set read timeout for `{address}`: {error}"),
            )
        })?;

        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            BrowserError::new(
                "net.transport.write_timeout_failed",
                format!("failed to set write timeout for `{address}`: {error}"),
            )
        })?;

        Ok(stream)
    }
}
