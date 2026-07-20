use std::sync::Arc;

use forge_config::Config;
use tokio::sync::RwLock;

pub struct ConfigService {
    config: Config,
}

impl ConfigService {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn handle(&self) -> ConfigHandle {
        ConfigHandle {
            workspace: Arc::new(RwLock::new(self.config.clone())),
        }
    }
}

#[derive(Clone)]
pub struct ConfigHandle {
    workspace: Arc<RwLock<Config>>,
}
