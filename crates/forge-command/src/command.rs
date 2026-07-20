use crate::{CommandDescriptor, CommandError};

pub trait Command: Send + Sync {
    fn descriptor(&self) -> &'static CommandDescriptor;

    fn execute(&self) -> Result<(), CommandError>;
}
