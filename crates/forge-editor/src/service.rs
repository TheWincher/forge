use std::{collections::HashMap, path::PathBuf, sync::Arc};

use forge_workspace::DocumentId;
use tokio::sync::RwLock;

use crate::{buffer::DocumentBuffer, error::EditorError, snapshot::DocumentBufferSnapshot};

pub struct EditorService {
    buffers: Arc<RwLock<HashMap<DocumentId, DocumentBuffer>>>,
}

impl EditorService {
    pub fn new() -> Self {
        Self {
            buffers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn handle(&self) -> EditorHandle {
        EditorHandle {
            buffers: Arc::clone(&self.buffers),
        }
    }
}

#[derive(Clone)]
pub struct EditorHandle {
    buffers: Arc<RwLock<HashMap<DocumentId, DocumentBuffer>>>,
}

impl EditorHandle {
    pub async fn open_buffer(
        &self,
        document_id: DocumentId,
        path: impl Into<PathBuf>,
    ) -> Result<(), EditorError> {
        let path = path.into();

        {
            let buffers = self.buffers.read().await;

            if buffers.contains_key(&document_id) {
                return Err(EditorError::BufferAlreadyOpen(document_id));
            }
        }

        let buffer = DocumentBuffer::load(document_id, path)?;
        let mut buffers = self.buffers.write().await;
        if buffers.contains_key(&document_id) {
            return Err(EditorError::BufferAlreadyOpen(document_id));
        }

        buffers.insert(document_id, buffer);

        Ok(())
    }

    pub async fn close_buffer(&self, document_id: DocumentId) -> Result<(), EditorError> {
        let mut buffers = self.buffers.write().await;

        buffers
            .remove(&document_id)
            .ok_or(EditorError::BufferNotOpen(document_id))?;

        Ok(())
    }

    pub async fn replace_content(
        &self,
        document_id: DocumentId,
        content: impl Into<String>,
    ) -> Result<(), EditorError> {
        let mut buffers = self.buffers.write().await;

        let buffer = buffers
            .get_mut(&document_id)
            .ok_or(EditorError::BufferNotOpen(document_id))?;

        buffer.replace_content(content);
        Ok(())
    }

    pub async fn save(&self, document_id: DocumentId) -> Result<(), EditorError> {
        let mut buffers = self.buffers.write().await;

        let buffer = buffers
            .get_mut(&document_id)
            .ok_or(EditorError::BufferNotOpen(document_id))?;

        buffer.save()?;

        Ok(())
    }

    pub async fn buffer(
        &self,
        document_id: DocumentId,
    ) -> Result<DocumentBufferSnapshot, EditorError> {
        let buffers = self.buffers.read().await;
        if let Some(buffer) = buffers.get(&document_id) {
            Ok(DocumentBufferSnapshot {
                document_id: buffer.document_id(),
                path: buffer.path().to_path_buf(),
                content: buffer.content().to_string(),
                version: buffer.version(),
                dirty: buffer.is_dirty(),
            })
        } else {
            Err(EditorError::BufferNotOpen(document_id))
        }
    }
}
