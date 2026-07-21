mod documents;
mod error;
mod event;
mod id;
mod state;
mod workspace;

pub use documents::Document;
pub use documents::DocumentId;
pub use error::WorkspaceError;
pub use event::ActiveDocumentChanged;
pub use event::DocumentClosed;
pub use event::DocumentOpened;
pub use id::WorkspaceId;
pub use state::WorkspaceState;
pub use workspace::Workspace;
