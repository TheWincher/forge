pub struct CommandService {}

impl CommandService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle(&self) -> CommandHandle {
        CommandHandle {}
    }
}

#[derive(Clone)]
pub struct CommandHandle {}
