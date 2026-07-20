use crate::{error::RuntimeError, event::AppEvent, runtime::RuntimeAction};

pub struct EventDispatcher;

impl EventDispatcher {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn dispatch(&self, event: AppEvent) -> Result<RuntimeAction, RuntimeError> {
        match event {
            AppEvent::ShutdownRequested => Ok(RuntimeAction::Stop),
        }
    }
}
