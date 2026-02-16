//! Privacy-first defaults and anti-tracking knobs.

const KNOWN_TRACKER_SUFFIXES: &[&str] = &[
    "doubleclick.net",
    "googlesyndication.com",
    "google-analytics.com",
    "googletagmanager.com",
    "facebook.net",
    "facebook.com",
];

/// Global privacy policy enabled at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyPolicy {
    pub block_third_party_cookies: bool,
    pub strip_referrer_cross_origin: bool,
    pub block_known_trackers: bool,
    pub fingerprinting_resistance: bool,
}

impl Default for PrivacyPolicy {
    fn default() -> Self {
        Self {
            block_third_party_cookies: true,
            strip_referrer_cross_origin: true,
            block_known_trackers: true,
            fingerprinting_resistance: true,
        }
    }
}

impl PrivacyPolicy {
    /// Returns true if this host should be blocked by tracker protection.
    pub fn should_block_host(&self, host: &str) -> bool {
        if !self.block_known_trackers {
            return false;
        }

        let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
        if normalized.is_empty() {
            return false;
        }

        KNOWN_TRACKER_SUFFIXES
            .iter()
            .any(|suffix| normalized == *suffix || normalized.ends_with(&format!(".{suffix}")))
    }
}

#[cfg(test)]
mod tests {
    use super::PrivacyPolicy;

    #[test]
    fn blocks_known_tracker_hosts() {
        let policy = PrivacyPolicy::default();
        assert!(policy.should_block_host("stats.google-analytics.com"));
        assert!(policy.should_block_host("doubleclick.net"));
    }

    #[test]
    fn ignores_hosts_when_tracker_blocking_disabled() {
        let mut policy = PrivacyPolicy::default();
        policy.block_known_trackers = false;
        assert!(!policy.should_block_host("doubleclick.net"));
    }
}
