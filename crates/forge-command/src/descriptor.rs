#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDescriptor {
    id: &'static str,
    title: &'static str,
}

impl CommandDescriptor {
    pub const fn new(id: &'static str, title: &'static str) -> Self {
        Self { id, title }
    }

    pub const fn id(&self) -> &'static str {
        self.id
    }

    pub const fn title(&self) -> &'static str {
        self.title
    }
}
