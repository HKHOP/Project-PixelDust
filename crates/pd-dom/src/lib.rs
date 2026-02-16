//! DOM tree data structures.

/// ID used to address nodes in the DOM arena.
pub type NodeId = u64;

/// Minimal document model for early parsing/layout integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub root: NodeId,
    pub node_count: u32,
    pub text_bytes: u32,
}

impl Document {
    pub fn empty() -> Self {
        Self {
            title: String::new(),
            root: 0,
            node_count: 0,
            text_bytes: 0,
        }
    }

    pub fn has_root(&self) -> bool {
        self.root != 0
    }
}
