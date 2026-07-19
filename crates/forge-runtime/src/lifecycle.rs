#[derive(Debug, PartialEq)]
pub enum RuntimeState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}
