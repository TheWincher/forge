use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceId(u64);

impl WorkspaceId {
    pub fn new() -> Self {
        Self(NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}
