use std::any::Any;

pub trait Event: Any + Send + Sync {}

impl<E> Event for E where E: Any + Send + Sync {}
