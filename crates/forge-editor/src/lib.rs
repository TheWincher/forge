use std::{
    fs,
    path::{Path, PathBuf},
};

use forge_runtime::{context::RuntimeContext, plugin::Plugin};
use thiserror::Error;

pub struct Buffer {
    path: PathBuf,
    content: String,
}

impl Buffer {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("failed to read {0}: {1}")]
    ReadFailed(PathBuf, std::io::Error),
}

pub struct Editor {
    buffer: Option<Buffer>,
}

impl Plugin for Editor {
    fn init(&mut self, context: &RuntimeContext) {
        match context.workspace() {
            Some(workspace) => {
                tracing::debug!("Have a workspace: {:?}", workspace.root());
            }
            None => {
                tracing::debug!("Haven't workspace");
            }
        }
    }
}

impl Editor {
    pub fn new() -> Self {
        Self { buffer: None }
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<(), EditorError> {
        let content =
            fs::read_to_string(&path).map_err(|err| EditorError::ReadFailed(path.clone(), err))?;

        self.buffer = Some(Buffer { path, content });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn write_temp_file(name: &str, contents: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("forge-editor-test-{}-{}", std::process::id(), name));
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn open_file_succeeds_and_fills_buffer() {
        let mut editor = Editor::new();
        let path = write_temp_file("open-succeeds.txt", "hello forge");

        editor.open_file(path.clone()).unwrap();

        let buffer = editor.buffer().unwrap();
        assert_eq!(buffer.path(), path.as_path());
        assert_eq!(buffer.content(), "hello forge");
        fs::remove_file(&path).ok();
    }

    #[test]
    fn open_file_missing_path_fails_and_keeps_buffer_empty() {
        let mut editor = Editor::new();
        let path = env::temp_dir().join("forge-editor-test-does-not-exist.txt");

        let error = editor.open_file(path.clone()).unwrap_err();

        assert!(matches!(error, EditorError::ReadFailed(p, _) if p == path));
        assert!(editor.buffer().is_none());
    }

    #[test]
    fn open_file_replaces_previous_buffer() {
        let mut editor = Editor::new();
        let first = write_temp_file("open-replaces-first.txt", "first");
        let second = write_temp_file("open-replaces-second.txt", "second");

        editor.open_file(first.clone()).unwrap();
        editor.open_file(second.clone()).unwrap();

        let buffer = editor.buffer().unwrap();
        assert_eq!(buffer.path(), second.as_path());
        assert_eq!(buffer.content(), "second");
        fs::remove_file(&first).ok();
        fs::remove_file(&second).ok();
    }
}
