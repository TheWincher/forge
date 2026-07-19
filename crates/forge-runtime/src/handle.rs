use tokio::sync::mpsc::Sender;

use crate::{error::RuntimeError, event::AppEvent};

#[derive(Clone)]
pub struct RuntimeHandle {
    sender: Sender<AppEvent>,
}

impl RuntimeHandle {
    pub fn new(sender: &Sender<AppEvent>) -> Self {
        Self {
            sender: sender.clone(),
        }
    }

    pub fn shutdown(&self) -> Result<(), RuntimeError> {
        self.sender
            .try_send(AppEvent::ShutdownRequested)
            .map_err(RuntimeError::SendEventFailed)?;

        Ok(())
    }
}
