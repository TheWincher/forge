use anyhow::{Ok, Result};
use forge_runtime::runtime::Runtime;

fn main() -> Result<()> {
    let mut runtime = Runtime::new()?;
    runtime.run()?;

    Ok(())
}
