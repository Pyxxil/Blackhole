use std::{
    collections::HashSet,
    fmt::Debug,
    path::Path,
    sync::{Arc, LazyLock},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::instrument;

use crate::{filter::List, schedule::Schedule, server::udp::Upstream};

pub static CONFIG: LazyLock<Arc<RwLock<Config>>> = LazyLock::new(Arc::default);
static CONFIG_FILE: LazyLock<Arc<RwLock<String>>> = LazyLock::new(Arc::default);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(alias = "upstream", rename(serialize = "upstream"))]
    pub upstreams: HashSet<Upstream>,
    #[serde(alias = "filter", rename(serialize = "filter"), default)]
    pub filters: Vec<List>,
    #[serde(alias = "schedule", rename(serialize = "schedule"), default)]
    pub schedules: Vec<Schedule>,
}

#[async_trait]
pub trait Load {
    ///
    /// Load a configuration profile, which could be something like
    /// a path (e.g. a configuration file), or CLI arguments
    ///
    /// # Errors
    /// This may error in several cases, which should be documented
    /// in the implementation.
    ///
    async fn load(self, config: &mut Config) -> std::io::Result<()>;
}

#[async_trait]
impl Load for &Path {
    ///
    /// Load a file (e.g. Configuration file)
    ///
    /// # Errors
    /// Should the file not exist in readable form, this will fail. If the file also
    /// isn't valid toml this will fail.
    ///
    #[instrument(level = "info", ret, err, skip(self, config), fields(file = self.to_str()))]
    async fn load(self, config: &mut Config) -> std::io::Result<()> {
        *CONFIG_FILE.write().await = self.to_string_lossy().to_string();

        let conf = std::fs::read_to_string(self)?;
        let conf: Config = toml::from_str(&conf)?;

        config.upstreams.extend(conf.upstreams);
        config.filters.extend(conf.filters);
        config.schedules.extend(conf.schedules);

        Ok(())
    }
}

impl Config {
    ///
    /// Load a configuration profile
    ///
    /// # Errors
    /// This can fail if the configuration profile fails to load,
    /// see [`Load`]
    ///
    pub async fn load<C: Load + Debug>(loader: C) -> std::io::Result<()> {
        let mut config = CONFIG.write().await;
        loader.load(&mut config).await?;

        Ok(())
    }

    ///
    /// Save the config to disk
    ///
    /// # Errors
    /// While this should be unlikely, it is possible for this to
    /// result in an error if:
    ///  - There is no disk space left
    ///  - The config file is not writable
    ///
    pub async fn save() -> std::io::Result<()> {
        std::fs::write(
            Path::new(&*CONFIG_FILE.read().await),
            toml::to_string(&*CONFIG.read().await).unwrap_or_default(),
        )
    }

    ///
    /// Retrieve a config variable from the global Configuration
    ///
    pub async fn get<F, T>(func: F) -> T
    where
        F: Fn(&Config) -> T,
    {
        func(&*CONFIG.read().await)
    }

    ///
    /// Set a config variable in the global Configuration
    ///
    /// Note that this also saves the configuration to a file every time
    ///
    /// # Errors
    /// This will result in an error if saving the config to a file does
    ///
    pub async fn set<F>(func: F) -> std::io::Result<()>
    where
        F: Fn(&mut Config),
    {
        func(&mut *CONFIG.write().await);
        Self::save().await
    }
}
