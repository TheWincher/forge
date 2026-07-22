use core::fmt;

#[derive(Debug)]
pub enum AppEvent {
    ShutdownRequested,
}

impl fmt::Display for AppEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ShutdownRequested")?;
        Ok(())
    }
}
