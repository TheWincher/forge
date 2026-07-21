use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::{Event, EventListener};

pub(crate) type ErasedListener = Arc<dyn Fn(&dyn Any) + Send + Sync>;

pub struct EventBus {
    listeners: HashMap<TypeId, Vec<ErasedListener>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    pub fn subscribe<E, L>(&mut self, listener: L)
    where
        E: Event,
        L: EventListener<E> + 'static,
    {
        let listener = Arc::new(listener);

        let erased_listener: ErasedListener = Arc::new(move |event| {
            let Some(event) = event.downcast_ref::<E>() else {
                return;
            };

            listener.on_event(event);
        });

        self.listeners
            .entry(TypeId::of::<E>())
            .or_default()
            .push(erased_listener);
    }

    pub fn publish<E>(&self, event: &E)
    where
        E: Event,
    {
        let Some(listeners) = self.listeners.get(&TypeId::of::<E>()) else {
            return;
        };

        for listener in listeners {
            listener(event);
        }
    }

    pub fn listener_count<E>(&self) -> usize
    where
        E: Event,
    {
        self.listeners.get(&TypeId::of::<E>()).map_or(0, Vec::len)
    }

    pub fn has_listeners<E>(&self) -> bool
    where
        E: Event,
    {
        self.listener_count::<E>() > 0
    }

    pub(crate) fn listeners_for<E>(&self) -> Vec<ErasedListener>
    where
        E: Event,
    {
        self.listeners
            .get(&TypeId::of::<E>())
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[derive(Debug)]
    struct TestEvent {
        value: usize,
    }

    #[derive(Debug)]
    struct OtherEvent;

    #[test]
    fn publishes_event_to_listener() {
        let mut bus = EventBus::new();
        let received = Arc::new(AtomicUsize::new(0));

        let received_by_listener = Arc::clone(&received);

        bus.subscribe::<TestEvent, _>(move |event: &TestEvent| {
            received_by_listener.store(event.value, Ordering::SeqCst);
        });

        bus.publish(&TestEvent { value: 42 });

        assert_eq!(received.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn supports_multiple_listeners() {
        let mut bus = EventBus::new();
        let calls = Arc::new(AtomicUsize::new(0));

        for _ in 0..2 {
            let calls = Arc::clone(&calls);

            bus.subscribe::<TestEvent, _>(move |_: &TestEvent| {
                calls.fetch_add(1, Ordering::SeqCst);
            });
        }

        bus.publish(&TestEvent { value: 42 });

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn does_not_notify_other_event_types() {
        let mut bus = EventBus::new();
        let calls = Arc::new(AtomicUsize::new(0));

        let listener_calls = Arc::clone(&calls);

        bus.subscribe::<OtherEvent, _>(move |_: &OtherEvent| {
            listener_calls.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish(&TestEvent { value: 42 });

        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn exposes_listener_count() {
        let mut bus = EventBus::new();

        bus.subscribe::<TestEvent, _>(|_: &TestEvent| {});

        assert_eq!(bus.listener_count::<TestEvent>(), 1);
        assert_eq!(bus.listener_count::<OtherEvent>(), 0);
        assert!(bus.has_listeners::<TestEvent>());
        assert!(!bus.has_listeners::<OtherEvent>());
    }
}
