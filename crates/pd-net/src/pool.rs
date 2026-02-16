//! Connection pooling contracts.

use crate::transport::BoxedIoStream;
use crate::url::BrowserUrl;
use crate::url::Scheme;
use std::collections::HashMap;
use std::collections::VecDeque;

/// Logical key used for pooling reusable connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub scheme: Scheme,
    pub host: String,
    pub port: u16,
}

impl ConnectionKey {
    pub fn from_url(url: &BrowserUrl) -> Self {
        Self {
            scheme: url.scheme(),
            host: url.host().to_owned(),
            port: url.port(),
        }
    }
}

/// Pool telemetry contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolStats {
    pub keys: usize,
    pub idle_connections: usize,
}

/// Connection pool contract used by network clients.
pub trait ConnectionPool {
    fn checkout(&mut self, key: &ConnectionKey) -> Option<BoxedIoStream>;
    fn checkin(&mut self, key: ConnectionKey, stream: BoxedIoStream);
    fn clear(&mut self);
    fn stats(&self) -> PoolStats;
}

/// In-memory idle connection pool with per-origin cap.
pub struct InMemoryConnectionPool {
    max_idle_per_key: usize,
    idle: HashMap<ConnectionKey, VecDeque<BoxedIoStream>>,
}

impl InMemoryConnectionPool {
    pub fn new(max_idle_per_key: usize) -> Self {
        Self {
            max_idle_per_key,
            idle: HashMap::new(),
        }
    }
}

impl Default for InMemoryConnectionPool {
    fn default() -> Self {
        Self::new(8)
    }
}

impl ConnectionPool for InMemoryConnectionPool {
    fn checkout(&mut self, key: &ConnectionKey) -> Option<BoxedIoStream> {
        let queue = self.idle.get_mut(key)?;
        let stream = queue.pop_front();

        if queue.is_empty() {
            self.idle.remove(key);
        }

        stream
    }

    fn checkin(&mut self, key: ConnectionKey, stream: BoxedIoStream) {
        let queue = self.idle.entry(key).or_default();
        if queue.len() >= self.max_idle_per_key {
            return;
        }

        queue.push_back(stream);
    }

    fn clear(&mut self) {
        self.idle.clear();
    }

    fn stats(&self) -> PoolStats {
        let idle_connections = self.idle.values().map(VecDeque::len).sum();
        PoolStats {
            keys: self.idle.len(),
            idle_connections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionKey;
    use super::ConnectionPool;
    use super::InMemoryConnectionPool;
    use crate::transport::BoxedIoStream;
    use crate::url::BrowserUrl;
    use std::io::Read;
    use std::io::Write;

    struct StubStream;

    impl Read for StubStream {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Ok(0)
        }
    }

    impl Write for StubStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn boxed_stub_stream() -> BoxedIoStream {
        Box::new(StubStream)
    }

    #[test]
    fn checkin_and_checkout_roundtrip() {
        let url = BrowserUrl::parse("https://example.com/");
        assert!(url.is_ok());
        let key = ConnectionKey::from_url(&match url {
            Ok(value) => value,
            Err(error) => panic!("{error}"),
        });

        let mut pool = InMemoryConnectionPool::new(2);
        pool.checkin(key.clone(), boxed_stub_stream());
        pool.checkin(key.clone(), boxed_stub_stream());

        let stats = pool.stats();
        assert_eq!(stats.keys, 1);
        assert_eq!(stats.idle_connections, 2);

        assert!(pool.checkout(&key).is_some());
        assert!(pool.checkout(&key).is_some());
        assert!(pool.checkout(&key).is_none());
    }

    #[test]
    fn respects_per_key_limit() {
        let mut pool = InMemoryConnectionPool::new(1);
        let key = ConnectionKey {
            scheme: crate::url::Scheme::Https,
            host: "example.com".to_owned(),
            port: 443,
        };

        pool.checkin(key.clone(), boxed_stub_stream());
        pool.checkin(key.clone(), boxed_stub_stream());

        let stats = pool.stats();
        assert_eq!(stats.idle_connections, 1);
    }
}
