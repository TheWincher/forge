use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("Folder not found or it is a file: {0}")]
    FolderNotFoundOrFile(PathBuf),
}

impl Workspace {
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn open(root: PathBuf) -> Result<Workspace, WorkspaceError> {
        if !root.is_dir() {
            return Err(WorkspaceError::FolderNotFoundOrFile(root));
        }

        Ok(Self { root })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    #[test]
    fn open_existing_directory_succeeds() {
        let root = env::temp_dir();

        let workspace = Workspace::open(root.clone()).unwrap();

        assert_eq!(workspace.root(), root.as_path());
    }

    #[test]
    fn open_missing_path_fails() {
        let root = env::temp_dir().join("forge-workspace-test-does-not-exist");

        let error = Workspace::open(root.clone()).unwrap_err();

        assert!(matches!(error, WorkspaceError::FolderNotFoundOrFile(path) if path == root));
    }

    #[test]
    fn open_file_instead_of_directory_fails() {
        let root = env::temp_dir().join(format!("forge-workspace-test-file-{}", std::process::id()));
        fs::write(&root, "not a directory").unwrap();

        let error = Workspace::open(root.clone()).unwrap_err();

        assert!(matches!(error, WorkspaceError::FolderNotFoundOrFile(path) if path == root));
        fs::remove_file(&root).ok();
    }
}
