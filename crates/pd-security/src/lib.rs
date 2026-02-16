//! Security policy and sandbox defaults.

use pd_core::BrowserResult;

/// Central security policy for process and network hardening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPolicy {
    pub enforce_site_isolation: bool,
    pub enforce_strict_tls: bool,
    pub sandbox_renderer: bool,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            enforce_site_isolation: true,
            enforce_strict_tls: true,
            sandbox_renderer: true,
        }
    }
}

impl SecurityPolicy {
    pub fn validate(&self) -> BrowserResult<()> {
        if !self.sandbox_renderer {
            return Err(pd_core::BrowserError::new(
                "security.invalid_policy",
                "renderer sandbox must stay enabled",
            ));
        }

        Ok(())
    }
}
