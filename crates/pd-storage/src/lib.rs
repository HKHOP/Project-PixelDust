//! Storage subsystems (cookies, cache, local data) with partitioning defaults.

use pd_core::BrowserError;
use pd_core::BrowserResult;
use pd_privacy::PrivacyPolicy;
use pd_security::SecurityPolicy;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Durable storage configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageConfig {
    pub partition_by_top_level_site: bool,
    pub ephemeral_mode: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            partition_by_top_level_site: true,
            ephemeral_mode: false,
        }
    }
}

/// Entry point for all browser storage backends.
#[derive(Debug, Clone)]
pub struct StorageManager {
    pub config: StorageConfig,
    pub privacy: PrivacyPolicy,
    pub security: SecurityPolicy,
    persistent_root: Option<PathBuf>,
}

impl StorageManager {
    pub fn new(config: StorageConfig, privacy: PrivacyPolicy, security: SecurityPolicy) -> Self {
        Self {
            config,
            privacy,
            security,
            persistent_root: None,
        }
    }

    pub fn with_persistent_root(mut self, root: PathBuf) -> Self {
        self.persistent_root = Some(root);
        self
    }

    pub fn persistent_root(&self) -> Option<&Path> {
        self.persistent_root.as_deref()
    }

    pub fn set_partition_value(
        &self,
        top_level_site: &str,
        key: &str,
        value: &str,
    ) -> BrowserResult<()> {
        let path = self.partition_path(top_level_site)?;
        let mut map = read_partition_map(&path)?;
        map.insert(key.to_owned(), value.to_owned());
        write_partition_map(&path, &map)
    }

    pub fn get_partition_value(
        &self,
        top_level_site: &str,
        key: &str,
    ) -> BrowserResult<Option<String>> {
        let path = self.partition_path(top_level_site)?;
        let map = read_partition_map(&path)?;
        Ok(map.get(key).cloned())
    }

    pub fn remove_partition_value(&self, top_level_site: &str, key: &str) -> BrowserResult<()> {
        let path = self.partition_path(top_level_site)?;
        let mut map = read_partition_map(&path)?;
        map.remove(key);

        if map.is_empty() {
            if path.exists() {
                fs::remove_file(&path).map_err(|error| {
                    BrowserError::new(
                        "storage.partition_remove_failed",
                        format!(
                            "failed removing empty partition file `{}`: {error}",
                            path.display()
                        ),
                    )
                })?;
            }
            return Ok(());
        }

        write_partition_map(&path, &map)
    }

    fn partition_path(&self, top_level_site: &str) -> BrowserResult<PathBuf> {
        if self.config.ephemeral_mode {
            return Err(BrowserError::new(
                "storage.persistence_disabled",
                "persistent storage is disabled in ephemeral mode",
            ));
        }

        let root = self.persistent_root.as_ref().ok_or_else(|| {
            BrowserError::new(
                "storage.persistence_unconfigured",
                "persistent storage root is not configured",
            )
        })?;

        let partition = if self.config.partition_by_top_level_site {
            sanitize_partition_name(top_level_site)
        } else {
            "global".to_owned()
        };

        Ok(root.join("partitions").join(format!("{partition}.kv")))
    }
}

fn sanitize_partition_name(input: &str) -> String {
    let mut out = String::new();
    for ch in input.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "unknown".to_owned()
    } else {
        out
    }
}

fn read_partition_map(path: &Path) -> BrowserResult<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = fs::read_to_string(path).map_err(|error| {
        BrowserError::new(
            "storage.partition_read_failed",
            format!(
                "failed to read partition file `{}`: {error}",
                path.display()
            ),
        )
    })?;

    let mut map = BTreeMap::new();
    for (index, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let (key_hex, value_hex) = line.split_once('\t').ok_or_else(|| {
            BrowserError::new(
                "storage.partition_format_invalid",
                format!(
                    "invalid record format at `{}` line {}",
                    path.display(),
                    index + 1
                ),
            )
        })?;

        let key = decode_hex_string(key_hex)?;
        let value = decode_hex_string(value_hex)?;
        map.insert(key, value);
    }

    Ok(map)
}

fn write_partition_map(path: &Path, map: &BTreeMap<String, String>) -> BrowserResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            BrowserError::new(
                "storage.partition_dir_create_failed",
                format!(
                    "failed to create partition directory `{}`: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let mut encoded = String::new();
    for (key, value) in map {
        encoded.push_str(&encode_hex_string(key));
        encoded.push('\t');
        encoded.push_str(&encode_hex_string(value));
        encoded.push('\n');
    }

    fs::write(path, encoded).map_err(|error| {
        BrowserError::new(
            "storage.partition_write_failed",
            format!(
                "failed to write partition file `{}`: {error}",
                path.display()
            ),
        )
    })
}

fn encode_hex_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len().saturating_mul(2));
    for byte in value.as_bytes() {
        out.push(hex_char(byte >> 4));
        out.push(hex_char(byte & 0x0f));
    }
    out
}

fn decode_hex_string(value: &str) -> BrowserResult<String> {
    if !value.len().is_multiple_of(2) {
        return Err(BrowserError::new(
            "storage.partition_hex_invalid",
            "hex field length must be even",
        ));
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0_usize;
    while index < chars.len() {
        let high = decode_hex_nibble(chars[index])?;
        let low = decode_hex_nibble(chars[index + 1])?;
        bytes.push((high << 4) | low);
        index += 2;
    }

    String::from_utf8(bytes).map_err(|error| {
        BrowserError::new(
            "storage.partition_utf8_invalid",
            format!("partition field is not valid UTF-8: {error}"),
        )
    })
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + (value - 10)) as char,
        _ => '0',
    }
}

fn decode_hex_nibble(ch: char) -> BrowserResult<u8> {
    match ch {
        '0'..='9' => Ok((ch as u8) - b'0'),
        'a'..='f' => Ok((ch as u8) - b'a' + 10),
        'A'..='F' => Ok((ch as u8) - b'A' + 10),
        _ => Err(BrowserError::new(
            "storage.partition_hex_invalid",
            format!("invalid hex character `{ch}`"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::StorageConfig;
    use super::StorageManager;
    use pd_privacy::PrivacyPolicy;
    use pd_security::SecurityPolicy;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_storage_root() -> std::path::PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir().join(format!("pixeldust-storage-test-{stamp}"))
    }

    #[test]
    fn partition_value_roundtrip() {
        let root = temp_storage_root();
        let manager = StorageManager::new(
            StorageConfig::default(),
            PrivacyPolicy::default(),
            SecurityPolicy::default(),
        )
        .with_persistent_root(root.clone());

        let wrote = manager.set_partition_value("example.com", "session", "abc123");
        assert!(wrote.is_ok());

        let loaded = manager.get_partition_value("example.com", "session");
        assert_eq!(loaded, Ok(Some("abc123".to_owned())));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn ephemeral_mode_blocks_persistence() {
        let config = StorageConfig {
            partition_by_top_level_site: true,
            ephemeral_mode: true,
        };
        let manager =
            StorageManager::new(config, PrivacyPolicy::default(), SecurityPolicy::default())
                .with_persistent_root(temp_storage_root());

        let wrote = manager.set_partition_value("example.com", "k", "v");
        assert!(wrote.is_err());
        if let Err(error) = wrote {
            assert_eq!(error.code, "storage.persistence_disabled");
        }
    }
}
