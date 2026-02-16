//! DNS resolution contracts.

use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;

/// Name resolution abstraction.
pub trait DnsResolver {
    fn resolve(&self, host: &str, port: u16) -> BrowserResult<Vec<SocketAddr>>;
}

/// Uses the operating system resolver.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemDnsResolver;

impl DnsResolver for SystemDnsResolver {
    fn resolve(&self, host: &str, port: u16) -> BrowserResult<Vec<SocketAddr>> {
        let query = format!("{host}:{port}");
        let addresses: Vec<SocketAddr> = query
            .to_socket_addrs()
            .map_err(|error| {
                BrowserError::new(
                    "net.dns.resolve_failed",
                    format!("failed to resolve `{query}`: {error}"),
                )
            })?
            .collect();

        if addresses.is_empty() {
            return Err(BrowserError::new(
                "net.dns.no_results",
                format!("resolver returned no addresses for `{query}`"),
            ));
        }

        Ok(addresses)
    }
}
