use crate::context::RuntimeContext;

pub trait Plugin {
    fn init(&mut self, context: &RuntimeContext);
}
