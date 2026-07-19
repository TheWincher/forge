use anyhow::Result;
use forge_runtime::runtime::Runtime;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut runtime = Runtime::new()?;

    runtime.run()?;

    Ok(())
}
