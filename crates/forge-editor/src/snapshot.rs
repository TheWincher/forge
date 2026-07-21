use std::path::PathBuf;

use forge_workspace::DocumentId;

#[derive(Debug, Clone)]
pub struct DocumentBufferSnapshot {
    pub document_id: DocumentId,
    pub path: PathBuf,
    pub content: String,
    pub version: u64,
    pub dirty: bool,
}
