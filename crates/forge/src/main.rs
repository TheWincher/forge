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

    let mut runtime_task = tokio::task::spawn_blocking(move || runtime.run());

    runtime_handle.wait_until_running().await?;

    workspace.open(std::env::current_dir()?).await?;
    workspace.open_document("crates/forge/src/main.rs").await?;

    tokio::select! {
        result = &mut runtime_task => {
            result??;
        }

        result = tui.run() => {
            runtime_handle.shutdown()?;
            runtime_task.await??;
            result?;
        }
    }

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
