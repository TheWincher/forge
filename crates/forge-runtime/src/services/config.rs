use std::sync::Arc;

use forge_config::Config;
use tokio::sync::RwLock;

pub struct ConfigService {
    config: Arc<RwLock<Config>>,
}

impl ConfigService {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn config(&self) -> Config {
        self.config.read().await.clone()
    }
    pub fn handle(&self) -> ConfigHandle {
        ConfigHandle {
            config: Arc::clone(&self.config),
        }
    }
}

#[derive(Clone)]
pub struct ConfigHandle {
    config: Arc<RwLock<Config>>,
}

impl ConfigHandle {
    pub async fn current(&self) -> Config {
        self.config.read().await.clone()
    }
}
