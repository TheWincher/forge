use std::sync::{Arc, RwLock};

use crate::{Event, EventBus, EventListener};

#[derive(Clone)]
pub struct EventHandle {
    bus: Arc<RwLock<EventBus>>,
}

impl EventHandle {
    pub(crate) fn new(bus: Arc<RwLock<EventBus>>) -> Self {
        Self { bus }
    }

    pub fn subscribe<E, L>(&self, listener: L)
    where
        E: Event,
        L: EventListener<E> + 'static,
    {
        self.bus
            .write()
            .expect("event bus lock poisoned")
            .subscribe::<E, L>(listener);
    }

    pub fn publish<E>(&self, event: &E)
    where
        E: Event,
    {
        let listeners = {
            let bus = self.bus.read().expect("event bus lock poisoned");

            bus.listeners_for::<E>()
        };

        for listener in listeners {
            listener(event);
        }
    }

    pub fn listener_count<E>(&self) -> usize
    where
        E: Event,
    {
        self.bus
            .read()
            .expect("event bus lock poisoned")
            .listener_count::<E>()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use crate::EventService;

    #[derive(Debug)]
    struct TestEvent {
        value: usize,
    }

    #[derive(Debug)]
    struct OtherEvent;

    #[test]
    fn listener_can_publish_another_event() {
        let service = EventService::new();
        let handle = service.handle();

        let calls = Arc::new(AtomicUsize::new(0));

        let second_event_calls = Arc::clone(&calls);

        handle.subscribe::<OtherEvent, _>(move |_: &OtherEvent| {
            second_event_calls.fetch_add(1, Ordering::SeqCst);
        });

        let publisher = handle.clone();

        handle.subscribe::<TestEvent, _>(move |_: &TestEvent| {
            publisher.publish(&OtherEvent);
        });

        handle.publish(&TestEvent { value: 42 });

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
