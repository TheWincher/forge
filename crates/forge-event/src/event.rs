#[derive(Debug)]
pub enum AppEvent {
    Started,
    ShutdownRequested,
    ShutdownCompleted,
}
