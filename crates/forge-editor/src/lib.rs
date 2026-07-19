use forge_runtime::{context::RuntimeContext, plugin::Plugin};

pub struct Editor;

impl Editor {
    pub fn new() -> Self {
        Self {}
    }
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
