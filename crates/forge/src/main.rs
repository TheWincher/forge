use anyhow::Result;
use forge_editor::Editor;
use forge_runtime::runtime::Runtime;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut runtime = Runtime::new()?;

    runtime.register_plugin(Box::new(Editor::new()));

    runtime.run()?;

    Ok(())
}
