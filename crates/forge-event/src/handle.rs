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
