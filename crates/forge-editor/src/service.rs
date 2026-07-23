use std::{collections::HashMap, path::PathBuf, sync::Arc};

use forge_event::EventHandle;
use forge_workspace::{DocumentClosed, DocumentId, DocumentOpened};
use tokio::sync::RwLock;

use crate::{
    buffer::{BackspaceResult, DocumentBuffer},
    error::EditorError,
    snapshot::DocumentBufferSnapshot,
};

pub struct EditorService {
    buffers: Arc<RwLock<HashMap<DocumentId, DocumentBuffer>>>,
    events: EventHandle,
}

impl EditorService {
    pub fn new(events: EventHandle) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(HashMap::new())),
            events,
        }
    }

    pub fn start(&self) {
        let editor = self.handle();

        self.events
            .subscribe::<DocumentOpened, _>(move |event: &DocumentOpened| {
                let editor = editor.clone();
                let document_id = event.document_id;
                let path = event.path.clone();

                tokio::spawn(async move {
                    if let Err(error) = editor.open_buffer(document_id, path).await {
                        tracing::error!(
                            document_id = ?document_id,
                            error = %error,
                            "failed to open editor buffer"
                        );
                    }
                });
            });

        let editor = self.handle();

        self.events
            .subscribe::<DocumentClosed, _>(move |event: &DocumentClosed| {
                let editor = editor.clone();
                let document_id = event.document_id;

                tokio::spawn(async move {
                    match editor.close_buffer(document_id).await {
                        Ok(()) | Err(EditorError::BufferNotOpen(_)) => {}
                        Err(error) => {
                            tracing::error!(
                                document_id = ?document_id,
                                error = %error,
                                "failed to close editor buffer"
                            );
                        }
                    }
                });
            });
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

    pub async fn insert_character(
        &self,
        document_id: DocumentId,
        line: usize,
        column: usize,
        character: char,
    ) -> Result<bool, EditorError> {
        let mut buffers = self.buffers.write().await;

        let buffer = buffers
            .get_mut(&document_id)
            .ok_or(EditorError::BufferNotOpen(document_id))?;

        Ok(buffer.insert_charracter(line, column, character))
    }

    pub async fn backspace(
        &self,
        document_id: DocumentId,
        line: usize,
        column: usize,
    ) -> Result<BackspaceResult, EditorError> {
        let mut buffers = self.buffers.write().await;

        let buffer = buffers
            .get_mut(&document_id)
            .ok_or(EditorError::BufferNotOpen(document_id))?;

        Ok(buffer.backspace(line, column))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, future::Future, time::Duration};

    use forge_event::EventService;
    use forge_workspace::{DocumentClosed, DocumentId, DocumentOpened, WorkspaceId};
    use tempfile::tempdir;
    use tokio::time::{sleep, timeout};

    use super::EditorService;
    use crate::error::EditorError;

    async fn wait_until<F, Fut>(mut condition: F)
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = bool>,
    {
        timeout(Duration::from_secs(1), async {
            loop {
                if condition().await {
                    return;
                }

                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("condition was not satisfied before timeout");
    }

    #[tokio::test]
    async fn should_open_buffer_when_document_opened_event_is_published() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "Hello, Forge!").expect("failed to create temporary file");

        let events = EventService::new();
        let event_handle = events.handle();

        let editor_service = EditorService::new(event_handle.clone());
        editor_service.start();

        let editor = editor_service.handle();
        let document_id = DocumentId::new();

        event_handle.publish(&DocumentOpened {
            workspace_id: WorkspaceId::new(),
            document_id,
            path: path.clone(),
        });

        wait_until(|| {
            let editor = editor.clone();

            async move { editor.buffer(document_id).await.is_ok() }
        })
        .await;

        let buffer = editor
            .buffer(document_id)
            .await
            .expect("buffer should be open");

        assert_eq!(buffer.document_id, document_id);
        assert_eq!(buffer.path, path);
        assert_eq!(buffer.content, "Hello, Forge!");
        assert_eq!(buffer.version, 0);
        assert!(!buffer.dirty);
    }

    #[tokio::test]
    async fn should_close_buffer_when_document_closed_event_is_published() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "Hello, Forge!").expect("failed to create temporary file");

        let events = EventService::new();
        let event_handle = events.handle();

        let editor_service = EditorService::new(event_handle.clone());
        editor_service.start();

        let editor = editor_service.handle();
        let workspace_id = WorkspaceId::new();
        let document_id = DocumentId::new();

        event_handle.publish(&DocumentOpened {
            workspace_id,
            document_id,
            path,
        });

        wait_until(|| {
            let editor = editor.clone();

            async move { editor.buffer(document_id).await.is_ok() }
        })
        .await;

        event_handle.publish(&DocumentClosed {
            workspace_id,
            document_id,
        });

        wait_until(|| {
            let editor = editor.clone();

            async move {
                matches!(
                    editor.buffer(document_id).await,
                    Err(EditorError::BufferNotOpen(id)) if id == document_id
                )
            }
        })
        .await;
    }

    #[tokio::test]
    async fn should_ignore_document_closed_event_for_unknown_buffer() {
        let events = EventService::new();
        let event_handle = events.handle();

        let editor_service = EditorService::new(event_handle.clone());
        editor_service.start();

        let editor = editor_service.handle();
        let document_id = DocumentId::new();

        event_handle.publish(&DocumentClosed {
            workspace_id: WorkspaceId::new(),
            document_id,
        });

        tokio::task::yield_now().await;

        let result = editor.buffer(document_id).await;

        assert!(matches!(
            result,
            Err(EditorError::BufferNotOpen(id)) if id == document_id
        ));
    }

    #[tokio::test]
    async fn should_not_open_buffer_when_document_file_cannot_be_read() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("missing.txt");

        let events = EventService::new();
        let event_handle = events.handle();

        let editor_service = EditorService::new(event_handle.clone());
        editor_service.start();

        let editor = editor_service.handle();
        let document_id = DocumentId::new();

        event_handle.publish(&DocumentOpened {
            workspace_id: WorkspaceId::new(),
            document_id,
            path,
        });

        tokio::task::yield_now().await;

        assert!(matches!(
            editor.buffer(document_id).await,
            Err(EditorError::BufferNotOpen(id)) if id == document_id
        ));
    }
}
