use std::path::{Path, PathBuf};

use crate::{WorkspaceError, WorkspaceId, WorkspaceState};

#[derive(Debug, Clone)]
pub struct Workspace {
    id: WorkspaceId,
    root: PathBuf,
    state: WorkspaceState,
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
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

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

    fn temporary_file_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "forge-workspace-test-{}-{name}",
            std::process::id()
        ))
    }
}
