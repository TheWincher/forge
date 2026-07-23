use forge_workspace::DocumentId;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::EditorError;

pub enum BackspaceResult {
    Noop,
    CharacterDeleted,
    LinesJoined { line: usize, column: usize },
}

#[derive(Debug)]
pub struct DocumentBuffer {
    document_id: DocumentId,
    path: PathBuf,
    content: String,
    version: u64,
    dirty: bool,
}

impl DocumentBuffer {
    pub fn load(document_id: DocumentId, path: impl Into<PathBuf>) -> Result<Self, EditorError> {
        let path = path.into();
        if !path.is_file() {
            return Err(EditorError::FileNotFound(path));
        }

        let content = fs::read_to_string(&path).map_err(|err| EditorError::FailedToReadFile {
            path: path.clone(),
            source: err,
        })?;

        Ok(Self {
            document_id,
            path,
            content,
            version: 0,
            dirty: false,
        })
    }

    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn replace_content(&mut self, content: impl Into<String>) {
        let new_content = content.into();
        if self.content != new_content {
            self.content = new_content;
            self.version += 1;
            self.dirty = true;
        }
    }

    pub fn save(&mut self) -> Result<(), EditorError> {
        fs::write(&self.path, &self.content).map_err(|err| EditorError::FailedToSaveFile {
            path: self.path.clone(),
            source: err,
        })?;

        self.dirty = false;

        Ok(())
    }

    pub fn insert_charracter(&mut self, line: usize, column: usize, character: char) -> bool {
        let Some(byte_index) = Self::position_to_byte_index(&self.content, line, column) else {
            return false;
        };

        self.content.insert(byte_index, character);
        self.mark_modified();
        true
    }

    fn position_to_byte_index(
        content: &str,
        target_line: usize,
        target_column: usize,
    ) -> Option<usize> {
        let mut current_line = 0;
        let mut current_column = 0;

        for (byte_index, character) in content.char_indices() {
            if current_line == target_line && current_column == target_column {
                return Some(byte_index);
            }

            if character == '\n' {
                current_line += 1;
                current_column = 0;
            } else {
                current_column += 1;
            }
        }

        if current_line == target_line && current_column == target_column {
            Some(content.len())
        } else {
            None
        }
    }

    pub fn backspace(&mut self, line: usize, column: usize) -> BackspaceResult {
        if line == 0 && column == 0 {
            return BackspaceResult::Noop;
        }

        if column > 0 {
            let Some(byte_index) = Self::position_to_byte_index(&self.content, line, column) else {
                return BackspaceResult::Noop;
            };

            let previous_bytes_index = self.content[..byte_index]
                .char_indices()
                .next_back()
                .map(|(index, _)| index);

            let Some(previous_byte_index) = previous_bytes_index else {
                return BackspaceResult::Noop;
            };

            self.content
                .replace_range(previous_byte_index..byte_index, "");

            self.mark_modified();

            return BackspaceResult::CharacterDeleted;
        }

        let previous_line = line - 1;
        let previous_line_length = self
            .content
            .lines()
            .nth(previous_line)
            .map(str::chars)
            .map(Iterator::count)
            .unwrap_or(0);

        let Some(line_start) = Self::position_to_byte_index(&self.content, line, 0) else {
            return BackspaceResult::Noop;
        };

        let newline_index = line_start.saturating_sub(1);
        if self.content.as_bytes().get(newline_index) != Some(&b'\n') {
            return BackspaceResult::Noop;
        };

        self.content.remove(newline_index);
        self.mark_modified();

        BackspaceResult::LinesJoined {
            line: previous_line,
            column: previous_line_length,
        }
    }

    fn mark_modified(&mut self) {
        self.version += 1;
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use forge_workspace::DocumentId;
    use tempfile::tempdir;

    use super::DocumentBuffer;
    use crate::error::EditorError;

    #[test]
    fn should_load_document_buffer_from_file() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "Hello, Forge!").expect("failed to create temporary file");

        let document_id = DocumentId::new();
        let buffer =
            DocumentBuffer::load(document_id, &path).expect("failed to load document buffer");

        assert_eq!(buffer.document_id(), document_id);
        assert_eq!(buffer.path(), path.as_path());
        assert_eq!(buffer.content(), "Hello, Forge!");
        assert_eq!(buffer.version(), 0);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn should_return_file_not_found_when_loading_missing_file() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("missing.txt");

        let result = DocumentBuffer::load(DocumentId::new(), &path);

        assert!(matches!(
            result,
            Err(EditorError::FileNotFound(error_path))
                if error_path == path
        ));
    }

    #[test]
    fn should_replace_content() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "old content").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("new content");

        assert_eq!(buffer.content(), "new content");
        assert_eq!(buffer.version(), 1);
        assert!(buffer.is_dirty());
    }

    #[test]
    fn should_not_modify_buffer_when_content_is_identical() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "same content").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("same content");

        assert_eq!(buffer.content(), "same content");
        assert_eq!(buffer.version(), 0);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn should_increment_version_for_each_content_change() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "version zero").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("version one");
        buffer.replace_content("version two");

        assert_eq!(buffer.version(), 2);
        assert_eq!(buffer.content(), "version two");
        assert!(buffer.is_dirty());
    }

    #[test]
    fn should_save_content_to_file() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "old content").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("saved content");
        buffer.save().expect("failed to save buffer");

        let saved_content = fs::read_to_string(&path).expect("failed to read saved file");

        assert_eq!(saved_content, "saved content");
        assert!(!buffer.is_dirty());
        assert_eq!(buffer.version(), 1);
    }

    #[test]
    fn should_recreate_file_when_it_was_deleted_before_save() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "initial content").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("recreated content");

        fs::remove_file(&path).expect("failed to remove temporary file");

        buffer.save().expect("failed to recreate file");

        assert!(path.is_file());

        let saved_content = fs::read_to_string(&path).expect("failed to read recreated file");

        assert_eq!(saved_content, "recreated content");
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn should_keep_buffer_dirty_when_save_fails() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("document.txt");

        fs::write(&path, "initial content").expect("failed to create temporary file");

        let mut buffer =
            DocumentBuffer::load(DocumentId::new(), &path).expect("failed to load buffer");

        buffer.replace_content("modified content");

        fs::remove_dir_all(directory.path()).expect("failed to remove temporary directory");

        let result = buffer.save();

        assert!(matches!(
            result,
            Err(EditorError::FailedToSaveFile {
                path: error_path,
                ..
            }) if error_path == path
        ));

        assert!(buffer.is_dirty());
        assert_eq!(buffer.version(), 1);
        assert_eq!(buffer.content(), "modified content");
    }

    #[cfg(unix)]
    #[test]
    fn should_return_read_error_when_file_is_not_valid_utf8() {
        let directory = tempdir().expect("failed to create temporary directory");
        let path = directory.path().join("invalid-utf8.txt");

        fs::write(&path, [0xff, 0xfe, 0xfd]).expect("failed to create invalid UTF-8 file");

        let result = DocumentBuffer::load(DocumentId::new(), &path);

        assert!(matches!(
            result,
            Err(EditorError::FailedToReadFile {
                path: error_path,
                ..
            }) if error_path == path
        ));
    }
}
