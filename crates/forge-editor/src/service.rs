// use std::{collections::HashMap, sync::Arc};

// use forge_workspace::DocumentId;
// use tokio::sync::RwLock;

// pub struct EditorService {
//     buffers: Arc<RwLock<HashMap<DocumentId, DocumentBuffer>>>,
// }

// #[derive(Clone)]
// pub struct EditorHandle {
//     buffers: Arc<RwLock<HashMap<DocumentId, DocumentBuffer>>>,
// }

// impl EditorHandle {
//     pub async fn open_buffer(
//         &self,
//         document_id: DocumentId,
//         path: impl Into<PathBuf>,
//     ) -> Result<(), EditorError>;

//     pub async fn close_buffer(&self, document_id: DocumentId) -> Result<(), EditorError>;

//     pub async fn buffer(
//         &self,
//         document_id: DocumentId,
//     ) -> Result<Option<DocumentBufferSnapshot>, EditorError>;

//     pub async fn replace_content(
//         &self,
//         document_id: DocumentId,
//         content: impl Into<String>,
//     ) -> Result<(), EditorError>;

//     pub async fn save(&self, document_id: DocumentId) -> Result<(), EditorError>;
// }
