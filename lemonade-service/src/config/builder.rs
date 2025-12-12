//! Config Builder module
//!

use super::{Config, ConfigError, WorkerAddress};
use std::{path::PathBuf, time::Duration};

use constants::*;
use dotenv::dotenv;

/// Config builder
#[derive(Default)]
pub struct ConfigBuilder;

impl ConfigBuilder {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Config, ConfigError> {
        dotenv().ok();

        let listen_address = WorkerAddress::parse(
            std::env::var(WORKER_LISTEN_ADDRESS_ENV_KEY)
                .unwrap_or_else(|_| WORKER_LISTEN_ADDRESS_DEFAULT.to_string())
                .as_str(),
        )?;

        let service_name = std::env::var(WORKER_SERVICE_NAME_ENV_KEY)
            .unwrap_or_else(|_| WORKER_SERVICE_NAME_DEFAULT.to_string());

        let work_delay_ms = std::env::var(WORKER_WORK_DELAY_ENV_KEY)
            .unwrap_or_else(|_| WORKER_WORK_DELAY_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| ConfigError::Parse(format!("Invalid WORK_DELAY_MS: {}", e)))?;

        Ok(Config {
            listen_address,
            service_name,
            work_delay: Duration::from_millis(work_delay_ms),
        })
    }

    /// Load configuration from a file (supports JSON and TOML)
    pub fn from_file(path: Option<impl Into<PathBuf>>) -> Result<Config, ConfigError> {
        if let Some(path) = path {
            let path = path.into();
            if !path.exists() {
                return Err(ConfigError::FileNotFound(path));
            }

            let content = std::fs::read_to_string(&path)?;
            let extension =
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .ok_or_else(|| {
                        ConfigError::UnsupportedFormat(path.to_string_lossy().to_string())
                    })?;

            match extension.to_lowercase().as_str() {
                "json" => {
                    let config: Config = serde_json::from_str(&content)?;
                    Ok(config)
                }
                "toml" => {
                    let config: Config = toml::from_str(&content)?;
                    Ok(config)
                }
                _ => Err(ConfigError::UnsupportedFormat(
                    path.to_string_lossy().to_string(),
                )),
            }
        } else {
            Self::from_env()
        }
    }
}

mod constants {
    //! Constants module
    //!
    pub const WORKER_LISTEN_ADDRESS_ENV_KEY: &str = "LEMONADE_WORKER_LISTEN_ADDRESS";

    pub const WORKER_SERVICE_NAME_ENV_KEY: &str = "LEMONADE_WORKER_SERVICE_NAME";

    pub const WORKER_WORK_DELAY_ENV_KEY: &str = "LEMONADE_WORKER_WORK_DELAY_MS";

    pub const WORKER_LISTEN_ADDRESS_DEFAULT: &str = "127.0.0.1:50200";

    pub const WORKER_SERVICE_NAME_DEFAULT: &str = "lemonade-worker";

    pub const WORKER_WORK_DELAY_DEFAULT: u64 = 20;
}
