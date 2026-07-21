use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{Document, DocumentId, WorkspaceError, WorkspaceId, WorkspaceState};

#[derive(Debug, Clone)]
pub struct Workspace {
    id: WorkspaceId,
    root: PathBuf,
    state: WorkspaceState,
    documents: HashMap<DocumentId, Document>,
    active_document: Option<DocumentId>,
}

impl Workspace {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, WorkspaceError> {
        let root = root.into();

        if !root.is_dir() {
            return Err(WorkspaceError::FolderNotFoundOrFile(root));
        }

        Ok(Self {
            id: WorkspaceId::new(),
            root,
            state: WorkspaceState::Open,
            documents: HashMap::new(),
            active_document: None,
        })
    }

    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn state(&self) -> WorkspaceState {
        self.state
    }

    pub fn is_open(&self) -> bool {
        self.state == WorkspaceState::Open
    }

    pub fn close(&mut self) -> Result<(), WorkspaceError> {
        if self.state == WorkspaceState::Closed {
            return Err(WorkspaceError::AlreadyClosed);
        }

        self.state = WorkspaceState::Closed;

        Ok(())
    }

    pub fn is_document_open(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        self.documents
            .values()
            .any(|document| document.path() == path)
    }

    pub fn open_document(
        &mut self,
        path: impl Into<PathBuf>,
    ) -> Result<DocumentId, WorkspaceError> {
        let path = path.into();

        if !self.is_open() {
            return Err(WorkspaceError::WorkspaceClosed);
        }

        if !path.starts_with(&self.root) {
            return Err(WorkspaceError::DocumentOutsideWorkspace(path));
        }

        if !path.is_file() {
            return Err(WorkspaceError::FileNotFound(path));
        }

        if self.is_document_open(&path) {
            return Err(WorkspaceError::DocumentAlreadyOpen(path));
        }

        let document_id = DocumentId::new();
        let document = Document::new(document_id, path);

        self.documents.insert(document_id, document);

        if self.active_document.is_none() {
            self.active_document = Some(document_id);
        }

        Ok(document_id)
    }

    pub fn close_document(&mut self, id: DocumentId) -> Result<(), WorkspaceError> {
        if !self.is_open() {
            return Err(WorkspaceError::WorkspaceClosed);
        }

        self.documents
            .remove(&id)
            .ok_or(WorkspaceError::DocumentNotFound(id))?;

        if self.active_document == Some(id) {
            self.active_document = None;
        }

        Ok(())
    }

    pub fn set_active_document(&mut self, id: DocumentId) -> Result<(), WorkspaceError> {
        if !self.is_open() {
            return Err(WorkspaceError::WorkspaceClosed);
        }

        if self.document(id).is_none() {
            return Err(WorkspaceError::DocumentNotFound(id));
        }

        self.active_document = Some(id);

        Ok(())
    }

    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.active_document.and_then(|id| self.document(id))
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn opens_existing_directory() {
        let root = env::temp_dir();

        let workspace = Workspace::open(root.clone()).unwrap();

        assert_eq!(workspace.root(), root.as_path());
        assert!(workspace.is_open());
        assert_eq!(workspace.state(), WorkspaceState::Open);
    }

    #[test]
    fn assigns_different_ids_to_workspaces() {
        let root = env::temp_dir();

        let first = Workspace::open(root.clone()).unwrap();
        let second = Workspace::open(root).unwrap();

        assert_ne!(first.id(), second.id());
    }

    #[test]
    fn rejects_missing_directory() {
        let root = env::temp_dir().join(format!("forge-workspace-missing-{}", std::process::id()));

        let result = Workspace::open(root.clone());

        assert!(matches!(
            result,
            Err(WorkspaceError::FolderNotFoundOrFile(path))
                if path == root
        ));
    }

    #[test]
    fn rejects_file_as_workspace_root() {
        let root = temporary_file_path("not-a-directory");
        fs::write(&root, "content").unwrap();

        let result = Workspace::open(root.clone());

        assert!(matches!(
            result,
            Err(WorkspaceError::FolderNotFoundOrFile(path))
                if path == root
        ));

        fs::remove_file(root).ok();
    }

    #[test]
    fn closes_open_workspace() {
        let mut workspace = Workspace::open(env::temp_dir()).unwrap();

        workspace.close().unwrap();

        assert!(!workspace.is_open());
        assert_eq!(workspace.state(), WorkspaceState::Closed);
    }

    #[test]
    fn closing_workspace_twice_fails() {
        let mut workspace = Workspace::open(env::temp_dir()).unwrap();

        workspace.close().unwrap();
        let result = workspace.close();

        assert!(matches!(result, Err(WorkspaceError::AlreadyClosed)));
    }

    fn create_workspace() -> (TempDir, Workspace) {
        let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
        let workspace =
            Workspace::open(temp_dir.path()).expect("failed to open temporary workspace");
        (temp_dir, workspace)
    }
    fn create_file(temp_dir: &TempDir, name: &str) -> PathBuf {
        let path = temp_dir.path().join(name);
        fs::write(&path, "test content").expect("failed to create temporary file");
        path
    }
    #[test]
    fn should_open_workspace() {
        let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
        let workspace = Workspace::open(temp_dir.path()).expect("failed to open workspace");
        assert!(workspace.is_open());
        assert_eq!(workspace.state(), WorkspaceState::Open);
        assert_eq!(workspace.root(), temp_dir.path());
        assert_eq!(workspace.documents().count(), 0);
        assert!(workspace.active_document().is_none());
    }
    #[test]
    fn should_fail_to_open_workspace_when_folder_does_not_exist() {
        let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
        let missing_directory = temp_dir.path().join("missing");
        let result = Workspace::open(&missing_directory);
        assert!(
            matches!( result, Err(WorkspaceError::FolderNotFoundOrFile(path)) if path == missing_directory )
        );
    }
    #[test]
    fn should_fail_to_open_workspace_from_file() {
        let temp_dir = tempfile::tempdir().expect("failed to create temporary directory");
        let file_path = create_file(&temp_dir, "file.txt");
        let result = Workspace::open(&file_path);
        assert!(
            matches!( result, Err(WorkspaceError::FolderNotFoundOrFile(path)) if path == file_path )
        );
    }
    #[test]
    fn should_close_workspace() {
        let (_temp_dir, mut workspace) = create_workspace();
        workspace.close().expect("failed to close workspace");
        assert!(!workspace.is_open());
        assert_eq!(workspace.state(), WorkspaceState::Closed);
    }
    #[test]
    fn should_fail_when_closing_workspace_twice() {
        let (_temp_dir, mut workspace) = create_workspace();
        workspace.close().expect("failed to close workspace");
        let result = workspace.close();
        assert!(matches!(result, Err(WorkspaceError::AlreadyClosed)));
    }
    #[test]
    fn should_open_document() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        let document = workspace
            .document(document_id)
            .expect("opened document not found");
        assert_eq!(document.id(), document_id);
        assert_eq!(document.path(), file_path);
        assert_eq!(workspace.documents().count(), 1);
        assert!(workspace.is_document_open(&file_path));
    }
    #[test]
    fn should_set_first_opened_document_as_active() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        let active_document = workspace
            .active_document()
            .expect("active document not found");
        assert_eq!(active_document.id(), document_id);
        assert_eq!(active_document.path(), file_path);
    }
    #[test]
    fn should_not_replace_active_document_when_opening_another_document() {
        let (temp_dir, mut workspace) = create_workspace();
        let first_path = create_file(&temp_dir, "first.rs");
        let second_path = create_file(&temp_dir, "second.rs");
        let first_id = workspace
            .open_document(&first_path)
            .expect("failed to open first document");
        workspace
            .open_document(&second_path)
            .expect("failed to open second document");
        let active_document = workspace
            .active_document()
            .expect("active document not found");
        assert_eq!(active_document.id(), first_id);
    }
    #[test]
    fn should_fail_when_opening_document_twice() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        workspace
            .open_document(&file_path)
            .expect("failed to open document");
        let result = workspace.open_document(&file_path);
        assert!(
            matches!( result, Err(WorkspaceError::DocumentAlreadyOpen(path)) if path == file_path )
        );
    }
    #[test]
    fn should_fail_when_opening_missing_file() {
        let (temp_dir, mut workspace) = create_workspace();
        let missing_file = temp_dir.path().join("missing.rs");
        let result = workspace.open_document(&missing_file);
        assert!(
            matches!( result, Err(WorkspaceError::FileNotFound(path)) if path == missing_file )
        );
    }
    #[test]
    fn should_fail_when_opening_document_outside_workspace() {
        let (_temp_dir, mut workspace) = create_workspace();
        let external_directory =
            tempfile::tempdir().expect("failed to create external temporary directory");
        let external_file = create_file(&external_directory, "external.rs");
        let result = workspace.open_document(&external_file);
        assert!(
            matches!( result, Err(WorkspaceError::DocumentOutsideWorkspace(path)) if path == external_file )
        );
    }
    #[test]
    fn should_fail_when_opening_document_in_closed_workspace() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        workspace.close().expect("failed to close workspace");
        let result = workspace.open_document(&file_path);
        assert!(matches!(result, Err(WorkspaceError::WorkspaceClosed)));
    }
    #[test]
    fn should_return_document_by_id() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        let document = workspace.document(document_id);
        assert!(document.is_some());
        assert_eq!(document.expect("document not found").path(), file_path);
    }
    #[test]
    fn should_return_none_for_unknown_document_id() {
        let (_temp_dir, workspace) = create_workspace();
        let unknown_id = DocumentId::new();
        assert!(workspace.document(unknown_id).is_none());
    }
    #[test]
    fn should_return_all_open_documents() {
        let (temp_dir, mut workspace) = create_workspace();
        let first_path = create_file(&temp_dir, "first.rs");
        let second_path = create_file(&temp_dir, "second.rs");
        workspace
            .open_document(&first_path)
            .expect("failed to open first document");
        workspace
            .open_document(&second_path)
            .expect("failed to open second document");
        let paths: Vec<&Path> = workspace.documents().map(Document::path).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&first_path.as_path()));
        assert!(paths.contains(&second_path.as_path()));
    }
    #[test]
    fn should_set_active_document() {
        let (temp_dir, mut workspace) = create_workspace();
        let first_path = create_file(&temp_dir, "first.rs");
        let second_path = create_file(&temp_dir, "second.rs");
        workspace
            .open_document(&first_path)
            .expect("failed to open first document");
        let second_id = workspace
            .open_document(&second_path)
            .expect("failed to open second document");
        workspace
            .set_active_document(second_id)
            .expect("failed to set active document");
        let active_document = workspace
            .active_document()
            .expect("active document not found");
        assert_eq!(active_document.id(), second_id);
        assert_eq!(active_document.path(), second_path);
    }
    #[test]
    fn should_fail_when_setting_unknown_document_as_active() {
        let (_temp_dir, mut workspace) = create_workspace();
        let unknown_id = DocumentId::new();
        let result = workspace.set_active_document(unknown_id);
        assert!(matches!( result, Err(WorkspaceError::DocumentNotFound(id)) if id == unknown_id ));
    }
    #[test]
    fn should_fail_when_setting_active_document_in_closed_workspace() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        workspace.close().expect("failed to close workspace");
        let result = workspace.set_active_document(document_id);
        assert!(matches!(result, Err(WorkspaceError::WorkspaceClosed)));
    }
    #[test]
    fn should_close_document() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        workspace
            .close_document(document_id)
            .expect("failed to close document");
        assert!(workspace.document(document_id).is_none());
        assert!(!workspace.is_document_open(&file_path));
        assert_eq!(workspace.documents().count(), 0);
    }
    #[test]
    fn should_clear_active_document_when_closing_it() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        workspace
            .close_document(document_id)
            .expect("failed to close document");
        assert!(workspace.active_document().is_none());
    }
    #[test]
    fn should_keep_active_document_when_closing_another_document() {
        let (temp_dir, mut workspace) = create_workspace();
        let first_path = create_file(&temp_dir, "first.rs");
        let second_path = create_file(&temp_dir, "second.rs");
        let first_id = workspace
            .open_document(&first_path)
            .expect("failed to open first document");
        let second_id = workspace
            .open_document(&second_path)
            .expect("failed to open second document");
        workspace
            .close_document(second_id)
            .expect("failed to close second document");
        let active_document = workspace
            .active_document()
            .expect("active document not found");
        assert_eq!(active_document.id(), first_id);
    }
    #[test]
    fn should_fail_when_closing_unknown_document() {
        let (_temp_dir, mut workspace) = create_workspace();
        let unknown_id = DocumentId::new();
        let result = workspace.close_document(unknown_id);
        assert!(matches!( result, Err(WorkspaceError::DocumentNotFound(id)) if id == unknown_id ));
    }
    #[test]
    fn should_fail_when_closing_document_in_closed_workspace() {
        let (temp_dir, mut workspace) = create_workspace();
        let file_path = create_file(&temp_dir, "main.rs");
        let document_id = workspace
            .open_document(&file_path)
            .expect("failed to open document");
        workspace.close().expect("failed to close workspace");
        let result = workspace.close_document(document_id);
        assert!(matches!(result, Err(WorkspaceError::WorkspaceClosed)));
    }
    #[test]
    fn document_ids_should_be_unique() {
        let first_id = DocumentId::new();
        let second_id = DocumentId::new();
        assert_ne!(first_id, second_id);
    }
    #[test]
    fn document_id_value_should_return_inner_value() {
        let document_id = DocumentId::new();
        assert!(document_id.value() > 0);
    }

    fn temporary_file_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "forge-workspace-test-{}-{name}",
            std::process::id()
        ))
    }
}
