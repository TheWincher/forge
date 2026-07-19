use anyhow::{Ok, Result};
use forge_runtime::runtime::Runtime;

fn main() -> Result<()> {
    let runtime = Runtime::new()?;
    runtime.run();
    runtime.shutdown();

    Ok(())
}
