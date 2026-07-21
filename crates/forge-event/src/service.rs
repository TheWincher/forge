use std::sync::{Arc, RwLock};

use crate::{Event, EventBus, EventHandle, EventListener};

pub struct EventService {
    bus: Arc<RwLock<EventBus>>,
}

impl EventService {
    pub fn new() -> Self {
        Self {
            bus: Arc::new(RwLock::new(EventBus::new())),
        }
    }

    pub fn subscribe<E, L>(&self, listener: L)
    where
        E: Event,
        L: EventListener<E> + 'static,
    {
        self.handle().subscribe::<E, L>(listener);
    }

    pub fn publish<E>(&self, event: &E)
    where
        E: Event,
    {
        self.handle().publish(event);
    }

    pub fn handle(&self) -> EventHandle {
        EventHandle::new(Arc::clone(&self.bus))
    }
}

impl Default for EventService {
    fn default() -> Self {
        Self::new()
    }
}
