use tokio::sync::{mpsc::Sender, watch};

use crate::{error::RuntimeError, event::AppEvent, lifecycle::RuntimeState};

#[derive(Clone)]
pub struct RuntimeHandle {
    sender: Sender<AppEvent>,
    state_receiver: watch::Receiver<RuntimeState>,
}

impl RuntimeHandle {
    pub fn new(sender: &Sender<AppEvent>, state_receiver: watch::Receiver<RuntimeState>) -> Self {
        Self {
            sender: sender.clone(),
            state_receiver,
        }
    }

    pub async fn wait_until_running(&self) -> Result<(), RuntimeError> {
        let mut state_receiver = self.state_receiver.clone();

        loop {
            match *state_receiver.borrow_and_update() {
                RuntimeState::Running => return Ok(()),

                RuntimeState::Failed | RuntimeState::Stopped => {
                    return Err(RuntimeError::FailedToStart);
                }

                _ => {}
            }

            state_receiver
                .changed()
                .await
                .map_err(|_| RuntimeError::StateChannelClosed)?;
        }
    }

    pub fn shutdown(&self) -> Result<(), RuntimeError> {
        self.sender
            .try_send(AppEvent::ShutdownRequested)
            .map_err(RuntimeError::SendEventFailed)?;

        Ok(())
    }
}
