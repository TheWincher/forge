#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RuntimeState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}
