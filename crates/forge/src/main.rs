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

    let app = TuiApp::new(workspace.clone(), editor.clone());
    let mut tui = Tui::new(app)?;

    let mut runtime_task = tokio::task::spawn_blocking(move || runtime.run());

    tokio::select! {
        result = &mut runtime_task => {
            result??;
        }

        result = tui.run() => {
            tracing::debug!("tui stopped");
            runtime_handle.shutdown()?;
            tracing::debug!("shutdown requested");
            runtime_task.await??;
            tracing::debug!("runtime stopped");
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
