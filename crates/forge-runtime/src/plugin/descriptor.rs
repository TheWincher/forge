#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginDescriptor {
    id: &'static str,
    name: &'static str,
    version: &'static str,
}

impl PluginDescriptor {
    pub const fn new(id: &'static str, name: &'static str, version: &'static str) -> Self {
        Self { id, name, version }
    }

    pub const fn id(&self) -> &'static str {
        self.id
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn version(&self) -> &'static str {
        self.version
    }
}
