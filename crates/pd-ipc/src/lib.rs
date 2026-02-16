//! Process messaging and isolation channel definitions.

use pd_core::BrowserError;
use pd_core::BrowserResult;
use std::sync::mpsc;
use std::time::Duration;

const DEFAULT_MAX_MESSAGE_BYTES: usize = 64 * 1024;
const FRAME_PREFIX_BYTES: usize = 4;
const MESSAGE_TAG_PING: u8 = 1;
const MESSAGE_TAG_PONG: u8 = 2;
const MESSAGE_TAG_HEALTH_CHECK: u8 = 3;
const MESSAGE_TAG_HEALTH_REPORT: u8 = 4;
const MESSAGE_TAG_SHUTDOWN: u8 = 5;

/// Browser runtime process roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRole {
    Browser,
    Renderer,
    Network,
    Storage,
}

impl ProcessRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Renderer => "renderer",
            Self::Network => "network",
            Self::Storage => "storage",
        }
    }

    pub fn from_role_name(value: &str) -> Option<Self> {
        match value {
            "browser" => Some(Self::Browser),
            "renderer" => Some(Self::Renderer),
            "network" => Some(Self::Network),
            "storage" => Some(Self::Storage),
            _ => None,
        }
    }
}

/// Typed IPC message envelope used across process roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcMessage {
    Ping {
        request_id: u64,
    },
    Pong {
        request_id: u64,
    },
    HealthCheck {
        request_id: u64,
    },
    HealthReport {
        request_id: u64,
        role: ProcessRole,
        healthy: bool,
        detail: String,
    },
    Shutdown,
}

/// Defines how processes communicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelConfig {
    pub role: ProcessRole,
    pub max_message_bytes: usize,
}

impl ChannelConfig {
    pub fn hardened(role: ProcessRole) -> BrowserResult<Self> {
        let config = Self {
            role,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> BrowserResult<()> {
        if self.max_message_bytes == 0 {
            return Err(BrowserError::new(
                "ipc.max_message_bytes_invalid",
                "channel max_message_bytes must be greater than zero",
            ));
        }

        if self.max_message_bytes > (16 * 1024 * 1024) {
            return Err(BrowserError::new(
                "ipc.max_message_bytes_too_large",
                "channel max_message_bytes exceeds hard limit (16 MiB)",
            ));
        }

        Ok(())
    }
}

/// In-memory endpoint that applies framing and message-size checks.
pub struct LocalIpcEndpoint {
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    config: ChannelConfig,
}

impl LocalIpcEndpoint {
    pub fn role(&self) -> ProcessRole {
        self.config.role
    }

    pub fn send(&self, payload: &[u8]) -> BrowserResult<()> {
        let frame = encode_frame(payload, self.config.max_message_bytes)?;
        self.tx.send(frame).map_err(|error| {
            BrowserError::new(
                "ipc.send_failed",
                format!(
                    "failed to send message from {} endpoint: {error}",
                    self.config.role.as_str()
                ),
            )
        })
    }

    pub fn recv_timeout(&self, timeout: Duration) -> BrowserResult<Vec<u8>> {
        let frame = self.rx.recv_timeout(timeout).map_err(|error| {
            BrowserError::new(
                "ipc.recv_failed",
                format!(
                    "failed to receive message for {} endpoint: {error}",
                    self.config.role.as_str()
                ),
            )
        })?;
        decode_frame(&frame, self.config.max_message_bytes)
    }
}

/// Creates paired in-memory IPC endpoints.
pub fn local_channel_pair(
    left: ChannelConfig,
    right: ChannelConfig,
) -> BrowserResult<(LocalIpcEndpoint, LocalIpcEndpoint)> {
    left.validate()?;
    right.validate()?;

    let (left_to_right_tx, left_to_right_rx) = mpsc::channel();
    let (right_to_left_tx, right_to_left_rx) = mpsc::channel();

    Ok((
        LocalIpcEndpoint {
            tx: left_to_right_tx,
            rx: right_to_left_rx,
            config: left,
        },
        LocalIpcEndpoint {
            tx: right_to_left_tx,
            rx: left_to_right_rx,
            config: right,
        },
    ))
}

/// Encodes a payload as a length-prefixed frame.
pub fn encode_frame(payload: &[u8], max_message_bytes: usize) -> BrowserResult<Vec<u8>> {
    if payload.len() > max_message_bytes {
        return Err(BrowserError::new(
            "ipc.message_too_large",
            format!(
                "payload exceeds max_message_bytes ({} > {})",
                payload.len(),
                max_message_bytes
            ),
        ));
    }

    let len_u32 = u32::try_from(payload.len()).map_err(|_| {
        BrowserError::new(
            "ipc.message_too_large",
            "payload length does not fit in 32-bit frame prefix",
        )
    })?;

    let mut out = Vec::with_capacity(FRAME_PREFIX_BYTES + payload.len());
    out.extend_from_slice(&len_u32.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

/// Decodes a length-prefixed frame and validates payload size.
pub fn decode_frame(frame: &[u8], max_message_bytes: usize) -> BrowserResult<Vec<u8>> {
    if frame.len() < FRAME_PREFIX_BYTES {
        return Err(BrowserError::new(
            "ipc.frame_too_short",
            "frame is shorter than the 4-byte length prefix",
        ));
    }

    let mut len_bytes = [0_u8; FRAME_PREFIX_BYTES];
    len_bytes.copy_from_slice(&frame[..FRAME_PREFIX_BYTES]);
    let payload_len = u32::from_be_bytes(len_bytes) as usize;
    if payload_len > max_message_bytes {
        return Err(BrowserError::new(
            "ipc.message_too_large",
            format!(
                "decoded payload exceeds max_message_bytes ({} > {})",
                payload_len, max_message_bytes
            ),
        ));
    }

    let expected = FRAME_PREFIX_BYTES + payload_len;
    if frame.len() != expected {
        return Err(BrowserError::new(
            "ipc.frame_length_mismatch",
            format!(
                "frame length mismatch: expected {expected} bytes, got {}",
                frame.len()
            ),
        ));
    }

    Ok(frame[FRAME_PREFIX_BYTES..].to_vec())
}

/// Encodes a typed IPC message as a framed payload.
pub fn encode_message(message: &IpcMessage, max_message_bytes: usize) -> BrowserResult<Vec<u8>> {
    let payload = encode_message_payload(message)?;
    encode_frame(&payload, max_message_bytes)
}

/// Decodes a framed typed IPC message.
pub fn decode_message(frame: &[u8], max_message_bytes: usize) -> BrowserResult<IpcMessage> {
    let payload = decode_frame(frame, max_message_bytes)?;
    decode_message_payload(&payload)
}

fn encode_message_payload(message: &IpcMessage) -> BrowserResult<Vec<u8>> {
    match message {
        IpcMessage::Ping { request_id } => {
            let mut out = Vec::with_capacity(1 + 8);
            out.push(MESSAGE_TAG_PING);
            out.extend_from_slice(&request_id.to_be_bytes());
            Ok(out)
        }
        IpcMessage::Pong { request_id } => {
            let mut out = Vec::with_capacity(1 + 8);
            out.push(MESSAGE_TAG_PONG);
            out.extend_from_slice(&request_id.to_be_bytes());
            Ok(out)
        }
        IpcMessage::HealthCheck { request_id } => {
            let mut out = Vec::with_capacity(1 + 8);
            out.push(MESSAGE_TAG_HEALTH_CHECK);
            out.extend_from_slice(&request_id.to_be_bytes());
            Ok(out)
        }
        IpcMessage::HealthReport {
            request_id,
            role,
            healthy,
            detail,
        } => {
            let detail_bytes = detail.as_bytes();
            let detail_len = u16::try_from(detail_bytes.len()).map_err(|_| {
                BrowserError::new(
                    "ipc.message_detail_too_large",
                    format!(
                        "health-report detail exceeds 16-bit size limit ({} bytes)",
                        detail_bytes.len()
                    ),
                )
            })?;

            let mut out = Vec::with_capacity(1 + 8 + 1 + 1 + 2 + detail_bytes.len());
            out.push(MESSAGE_TAG_HEALTH_REPORT);
            out.extend_from_slice(&request_id.to_be_bytes());
            out.push(encode_role(*role));
            out.push(if *healthy { 1 } else { 0 });
            out.extend_from_slice(&detail_len.to_be_bytes());
            out.extend_from_slice(detail_bytes);
            Ok(out)
        }
        IpcMessage::Shutdown => Ok(vec![MESSAGE_TAG_SHUTDOWN]),
    }
}

fn decode_message_payload(payload: &[u8]) -> BrowserResult<IpcMessage> {
    if payload.is_empty() {
        return Err(BrowserError::new(
            "ipc.message_empty",
            "typed IPC payload is empty",
        ));
    }

    let mut offset = 0_usize;
    let tag = read_u8(payload, &mut offset, "tag")?;
    let message = match tag {
        MESSAGE_TAG_PING => IpcMessage::Ping {
            request_id: read_u64(payload, &mut offset, "request_id")?,
        },
        MESSAGE_TAG_PONG => IpcMessage::Pong {
            request_id: read_u64(payload, &mut offset, "request_id")?,
        },
        MESSAGE_TAG_HEALTH_CHECK => IpcMessage::HealthCheck {
            request_id: read_u64(payload, &mut offset, "request_id")?,
        },
        MESSAGE_TAG_HEALTH_REPORT => {
            let request_id = read_u64(payload, &mut offset, "request_id")?;
            let role = decode_role(read_u8(payload, &mut offset, "role")?)?;
            let healthy = match read_u8(payload, &mut offset, "healthy")? {
                0 => false,
                1 => true,
                other => {
                    return Err(BrowserError::new(
                        "ipc.message_field_invalid",
                        format!("invalid health flag `{other}`; expected 0 or 1"),
                    ));
                }
            };
            let detail = read_string_u16(payload, &mut offset, "detail")?;
            IpcMessage::HealthReport {
                request_id,
                role,
                healthy,
                detail,
            }
        }
        MESSAGE_TAG_SHUTDOWN => IpcMessage::Shutdown,
        other => {
            return Err(BrowserError::new(
                "ipc.message_tag_unknown",
                format!("unknown typed IPC message tag `{other}`"),
            ));
        }
    };

    if offset != payload.len() {
        return Err(BrowserError::new(
            "ipc.message_trailing_bytes",
            format!(
                "typed IPC payload has trailing bytes (decoded {offset} of {})",
                payload.len()
            ),
        ));
    }

    Ok(message)
}

fn encode_role(role: ProcessRole) -> u8 {
    match role {
        ProcessRole::Browser => 1,
        ProcessRole::Renderer => 2,
        ProcessRole::Network => 3,
        ProcessRole::Storage => 4,
    }
}

fn decode_role(raw: u8) -> BrowserResult<ProcessRole> {
    match raw {
        1 => Ok(ProcessRole::Browser),
        2 => Ok(ProcessRole::Renderer),
        3 => Ok(ProcessRole::Network),
        4 => Ok(ProcessRole::Storage),
        _ => Err(BrowserError::new(
            "ipc.message_role_invalid",
            format!("invalid role code `{raw}` in typed IPC payload"),
        )),
    }
}

fn read_u8(payload: &[u8], offset: &mut usize, field: &str) -> BrowserResult<u8> {
    if *offset >= payload.len() {
        return Err(BrowserError::new(
            "ipc.message_truncated",
            format!("missing `{field}` in typed IPC payload"),
        ));
    }

    let value = payload[*offset];
    *offset += 1;
    Ok(value)
}

fn read_u16(payload: &[u8], offset: &mut usize, field: &str) -> BrowserResult<u16> {
    let bytes = read_exact(payload, offset, 2, field)?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

fn read_u64(payload: &[u8], offset: &mut usize, field: &str) -> BrowserResult<u64> {
    let bytes = read_exact(payload, offset, 8, field)?;
    Ok(u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

fn read_string_u16(payload: &[u8], offset: &mut usize, field: &str) -> BrowserResult<String> {
    let len = usize::from(read_u16(payload, offset, field)?);
    let bytes = read_exact(payload, offset, len, field)?;
    String::from_utf8(bytes.to_vec()).map_err(|error| {
        BrowserError::new(
            "ipc.message_utf8_invalid",
            format!("typed IPC field `{field}` is not valid UTF-8: {error}"),
        )
    })
}

fn read_exact<'a>(
    payload: &'a [u8],
    offset: &mut usize,
    len: usize,
    field: &str,
) -> BrowserResult<&'a [u8]> {
    let end = offset.saturating_add(len);
    if end > payload.len() {
        return Err(BrowserError::new(
            "ipc.message_truncated",
            format!("typed IPC payload ended while reading `{field}` (need {len} bytes)"),
        ));
    }

    let out = &payload[*offset..end];
    *offset = end;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::ChannelConfig;
    use super::IpcMessage;
    use super::ProcessRole;
    use super::decode_frame;
    use super::decode_message;
    use super::encode_frame;
    use super::encode_message;
    use super::local_channel_pair;
    use std::time::Duration;

    #[test]
    fn role_roundtrip_from_str() {
        assert_eq!(
            ProcessRole::from_role_name("renderer"),
            Some(ProcessRole::Renderer)
        );
        assert_eq!(ProcessRole::Renderer.as_str(), "renderer");
        assert_eq!(ProcessRole::from_role_name("invalid"), None);
    }

    #[test]
    fn frame_roundtrip() {
        let encoded = encode_frame(b"hello", 64);
        assert!(encoded.is_ok());
        let encoded = encoded.unwrap_or_else(|_| unreachable!());
        let decoded = decode_frame(&encoded, 64);
        assert_eq!(decoded, Ok(b"hello".to_vec()));
    }

    #[test]
    fn local_channel_sends_and_receives() {
        let left = ChannelConfig::hardened(ProcessRole::Browser);
        assert!(left.is_ok());
        let right = ChannelConfig::hardened(ProcessRole::Renderer);
        assert!(right.is_ok());
        let pair = local_channel_pair(
            left.unwrap_or_else(|_| unreachable!()),
            right.unwrap_or_else(|_| unreachable!()),
        );
        assert!(pair.is_ok());
        let (browser, renderer) = pair.unwrap_or_else(|_| unreachable!());

        let sent = browser.send(b"ping");
        assert!(sent.is_ok());

        let received = renderer.recv_timeout(Duration::from_secs(1));
        assert_eq!(received, Ok(b"ping".to_vec()));
    }

    #[test]
    fn typed_message_roundtrip() {
        let encoded = encode_message(
            &IpcMessage::HealthReport {
                request_id: 42,
                role: ProcessRole::Renderer,
                healthy: true,
                detail: "ready".to_owned(),
            },
            4096,
        );
        assert!(encoded.is_ok());

        let decoded = decode_message(&encoded.unwrap_or_else(|_| unreachable!()), 4096);
        assert_eq!(
            decoded,
            Ok(IpcMessage::HealthReport {
                request_id: 42,
                role: ProcessRole::Renderer,
                healthy: true,
                detail: "ready".to_owned(),
            })
        );
    }

    #[test]
    fn typed_message_rejects_unknown_tag() {
        let frame = encode_frame(&[99], 64);
        assert!(frame.is_ok());
        let decoded = decode_message(&frame.unwrap_or_else(|_| unreachable!()), 64);
        assert!(decoded.is_err());
        if let Err(error) = decoded {
            assert_eq!(error.code, "ipc.message_tag_unknown");
        }
    }
}
