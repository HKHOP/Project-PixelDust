//! Shared primitives used across PixelDust crates.

use core::fmt;

/// Result alias used across the workspace.
pub type BrowserResult<T> = Result<T, BrowserError>;

/// Top-level error type for early scaffolding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserError {
    pub code: &'static str,
    pub message: String,
}

impl BrowserError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for BrowserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for BrowserError {}
