use crate::{context::RuntimeContext, plugin::Plugin};

pub struct PluginService {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginService {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn init_all(&mut self, context: &RuntimeContext) {
        for plugin in &mut self.plugins {
            plugin.init(context);
        }
    }
}
