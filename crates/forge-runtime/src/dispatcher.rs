use crate::{error::RuntimeError, event::AppEvent, runtime::RuntimeAction};

pub struct EventDispatcher;

impl EventDispatcher {
    pub async fn dispatch(event: AppEvent) -> Result<RuntimeAction, RuntimeError> {
        match event {
            AppEvent::ShutdownRequested => Ok(RuntimeAction::Stop),
            AppEvent::Started => Ok(RuntimeAction::Continue),
            AppEvent::ShutdownCompleted => Ok(RuntimeAction::Continue),
        }
    }
}
