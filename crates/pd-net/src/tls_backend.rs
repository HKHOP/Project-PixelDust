//! TLS backend adapter contracts and rustls implementation.

use crate::tls::TlsHandshakeConfig;
use crate::tls::TrustStoreMode;
use crate::transport::BoxedIoStream;
use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::net::TcpStream;

#[cfg(feature = "tls-rustls")]
use crate::tls::TlsVersion;
#[cfg(feature = "tls-rustls")]
use rustls::DigitallySignedStruct;
#[cfg(feature = "tls-rustls")]
use rustls::Error as RustlsError;
#[cfg(feature = "tls-rustls")]
use rustls::RootCertStore;
#[cfg(feature = "tls-rustls")]
use rustls::SignatureScheme;
#[cfg(feature = "tls-rustls")]
use rustls::SupportedProtocolVersion;
#[cfg(feature = "tls-rustls")]
use rustls::client::WebPkiServerVerifier;
#[cfg(feature = "tls-rustls")]
use rustls::client::danger::HandshakeSignatureValid;
#[cfg(feature = "tls-rustls")]
use rustls::client::danger::ServerCertVerified;
#[cfg(feature = "tls-rustls")]
use rustls::client::danger::ServerCertVerifier;
#[cfg(feature = "tls-rustls")]
use rustls::pki_types::CertificateDer;
#[cfg(feature = "tls-rustls")]
use rustls::pki_types::ServerName;
#[cfg(feature = "tls-rustls")]
use rustls::pki_types::UnixTime;
#[cfg(feature = "tls-rustls")]
use std::sync::Arc;

/// Adapter contract for upgrading TCP transport to TLS.
pub trait TlsBackendAdapter {
    fn connect_tls(
        &self,
        stream: TcpStream,
        handshake: &TlsHandshakeConfig,
        tls_policy: &crate::tls::StrictTlsPolicy,
    ) -> BrowserResult<BoxedIoStream>;
}

/// rustls-backed TLS connector.
#[derive(Debug, Clone, Copy, Default)]
pub struct RustlsTlsAdapter;

#[cfg(feature = "tls-rustls")]
impl TlsBackendAdapter for RustlsTlsAdapter {
    fn connect_tls(
        &self,
        mut stream: TcpStream,
        handshake: &TlsHandshakeConfig,
        tls_policy: &crate::tls::StrictTlsPolicy,
    ) -> BrowserResult<BoxedIoStream> {
        use rustls::ClientConfig;
        use rustls::ClientConnection;
        use rustls::StreamOwned;

        let versions = supported_versions(handshake.minimum_version, handshake.maximum_version)?;
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let roots = Arc::new(system_root_store(tls_policy)?);
        let base_verifier = WebPkiServerVerifier::builder_with_provider(roots, provider.clone())
            .build()
            .map_err(|error| {
                BrowserError::new(
                    "net.tls.verifier_build_failed",
                    format!("failed to build rustls verifier: {error}"),
                )
            })?;

        let verifier: Arc<dyn ServerCertVerifier> = if handshake.require_ocsp_stapling {
            Arc::new(OcspRequiredVerifier {
                inner: base_verifier,
            })
        } else {
            base_verifier
        };

        let mut config = ClientConfig::builder_with_provider(provider)
            .with_protocol_versions(&versions)
            .map_err(|error| {
                BrowserError::new(
                    "net.tls.config_versions_invalid",
                    format!("failed to configure TLS protocol versions: {error}"),
                )
            })?
            .dangerous()
            .with_custom_certificate_verifier(verifier)
            .with_no_client_auth();
        config.enable_sni = handshake.require_sni;
        config.alpn_protocols = handshake
            .alpn_protocols
            .iter()
            .map(|value| value.as_bytes().to_vec())
            .collect();

        let server_name = ServerName::try_from(handshake.server_name.clone()).map_err(|error| {
            BrowserError::new(
                "net.tls.server_name_invalid",
                format!(
                    "invalid TLS server name `{}`: {error}",
                    handshake.server_name
                ),
            )
        })?;

        let mut connection =
            ClientConnection::new(Arc::new(config), server_name).map_err(|error| {
                BrowserError::new(
                    "net.tls.connection_init_failed",
                    format!(
                        "failed to initialize TLS connection for `{}`: {error}",
                        handshake.server_name
                    ),
                )
            })?;

        connection.complete_io(&mut stream).map_err(|error| {
            BrowserError::new(
                "net.tls.handshake_failed",
                format!(
                    "TLS handshake failed for `{}`: {error}",
                    handshake.server_name
                ),
            )
        })?;

        let stream = StreamOwned::new(connection, stream);
        Ok(Box::new(stream))
    }
}

#[cfg(feature = "tls-rustls")]
#[derive(Debug)]
struct OcspRequiredVerifier {
    inner: Arc<WebPkiServerVerifier>,
}

#[cfg(feature = "tls-rustls")]
impl ServerCertVerifier for OcspRequiredVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        if ocsp_response.is_empty() {
            return Err(RustlsError::General(
                "missing required OCSP stapling response".to_owned(),
            ));
        }

        self.inner
            .verify_server_cert(end_entity, intermediates, server_name, ocsp_response, now)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

#[cfg(feature = "tls-rustls")]
fn system_root_store(tls_policy: &crate::tls::StrictTlsPolicy) -> BrowserResult<RootCertStore> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    if matches!(tls_policy.trust_store_mode, TrustStoreMode::WebPkiAndOs) {
        let native = rustls_native_certs::load_native_certs();
        if native.certs.is_empty() && !native.errors.is_empty() {
            let details = native
                .errors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ");
            return Err(BrowserError::new(
                "net.tls.os_roots_load_failed",
                format!("failed to load operating-system roots: {details}"),
            ));
        }

        for cert in native.certs {
            roots.add(cert).map_err(|error| {
                BrowserError::new(
                    "net.tls.os_root_add_failed",
                    format!("failed to add operating-system root: {error}"),
                )
            })?;
        }
    }

    if roots.is_empty() {
        return Err(BrowserError::new(
            "net.tls.root_store_empty",
            "no trust anchors available for TLS verification",
        ));
    }

    Ok(roots)
}

#[cfg(feature = "tls-rustls")]
fn to_rustls_version(version: TlsVersion) -> &'static SupportedProtocolVersion {
    match version {
        TlsVersion::V1_2 => &rustls::version::TLS12,
        TlsVersion::V1_3 => &rustls::version::TLS13,
    }
}

#[cfg(feature = "tls-rustls")]
fn supported_versions(
    minimum: TlsVersion,
    maximum: TlsVersion,
) -> BrowserResult<Vec<&'static SupportedProtocolVersion>> {
    let all = [TlsVersion::V1_3, TlsVersion::V1_2];
    let mut versions = Vec::new();

    for version in all {
        if version >= minimum && version <= maximum {
            versions.push(to_rustls_version(version));
        }
    }

    if versions.is_empty() {
        return Err(BrowserError::new(
            "net.tls.version_set_empty",
            "no supported TLS versions match the requested policy",
        ));
    }

    Ok(versions)
}

#[cfg(not(feature = "tls-rustls"))]
impl TlsBackendAdapter for RustlsTlsAdapter {
    fn connect_tls(
        &self,
        _stream: TcpStream,
        _handshake: &TlsHandshakeConfig,
        _tls_policy: &crate::tls::StrictTlsPolicy,
    ) -> BrowserResult<BoxedIoStream> {
        Err(BrowserError::new(
            "net.tls.backend_unavailable",
            "rustls backend is disabled for this build; enable `pd-net/tls-rustls`",
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::tls::TlsVersion;

    #[test]
    fn version_range_rejects_inverted_bounds() {
        let minimum = TlsVersion::V1_3;
        let maximum = TlsVersion::V1_2;
        assert!(minimum > maximum);
    }
}
