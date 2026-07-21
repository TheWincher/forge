use crate::Event;

pub trait EventListener<E>: Send + Sync
where
    E: Event,
{
    fn on_event(&self, event: &E);
}

impl<E, F> EventListener<E> for F
where
    E: Event,
    F: Fn(&E) + Send + Sync,
{
    fn on_event(&self, event: &E) {
        self(event);
    }
}
