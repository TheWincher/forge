use std::fs::OpenOptions;

use forge_runtime::runtime::Runtime;
use forge_tui::Tui;
use forge_tui::TuiApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let mut runtime = Runtime::new()?;
    let runtime_handle = runtime.handle();

    let workspace = runtime.context().services().workspace().clone();
    let editor = runtime.context().services().editor().clone();

    let app = TuiApp::new(workspace.clone(), editor);
    let mut tui = Tui::new(app)?;

    let runtime_task = tokio::task::spawn_blocking(move || runtime.run());

    let application_result: anyhow::Result<()> = async {
        runtime_handle.wait_until_running().await?;

        let workspace_root = std::env::current_dir()?;

        workspace.open(workspace_root.clone()).await?;
        workspace.open_document("crates/forge/src/main.rs").await?;

        tui.run().await?;

        Ok(())
    }
    .await;

    let shutdown_result = runtime_handle.shutdown();
    if let Err(error) = &shutdown_result {
        tracing::error!(?error, "Failed to request runtime shutdown");
    }

    let runtime_result = runtime_task.await;
    if let Err(error) = &runtime_result {
        tracing::error!(?error, "Runtime task failed to join");
    }

    application_result?;

    shutdown_result?;
    runtime_result??;

    Ok(())
}

fn init_logging() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("forge.log")
        .unwrap();

    tracing_subscriber::fmt().with_writer(file).init();
}
