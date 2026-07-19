use tokio::signal::unix::{SignalKind, signal};

use crate::{error::RuntimeError, handle::RuntimeHandle};

pub async fn wait_for_shutdown(handle: RuntimeHandle) -> Result<(), RuntimeError> {
    let mut sigterm = signal(SignalKind::terminate())?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received Ctrl+C");
        }

        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM");
        }
    }

    handle.shutdown()?;

    Ok(())
}
