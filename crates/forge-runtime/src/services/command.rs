pub struct CommandService {}

impl CommandService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(&self) -> CommandHandle {
        CommandHandle {}
    }
}

impl Default for CommandService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct CommandHandle {}
