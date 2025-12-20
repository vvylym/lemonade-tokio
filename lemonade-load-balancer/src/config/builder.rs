//! Config Builder module
//!
//! Provides methods to load configuration from files or environment variables
use crate::prelude::*;
use crate::strategy::error::StrategyError;
use std::path::PathBuf;
use std::time::Duration;

use constants::*;
use dotenvy::dotenv;

/// Config builder
#[derive(Default)]
pub struct ConfigBuilder;

impl ConfigBuilder {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Config, ConfigError> {
        dotenv().ok();

        // Runtime config
        let metrics_cap = std::env::var(LB_METRICS_CAP_ENV_KEY)
            .unwrap_or_else(|_| LB_METRICS_CAP_DEFAULT.to_string())
            .parse::<usize>()
            .map_err(|e| {
                ConfigError::Parse(format!("Invalid {}: {}", LB_METRICS_CAP_ENV_KEY, e))
            })?;

        let health_cap = std::env::var(LB_HEALTH_CAP_ENV_KEY)
            .unwrap_or_else(|_| LB_HEALTH_CAP_DEFAULT.to_string())
            .parse::<usize>()
            .map_err(|e| {
                ConfigError::Parse(format!("Invalid {}: {}", LB_HEALTH_CAP_ENV_KEY, e))
            })?;

        let drain_timeout_millis = std::env::var(LB_DRAIN_TIMEOUT_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_DRAIN_TIMEOUT_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_DRAIN_TIMEOUT_MS_ENV_KEY, e
                ))
            })?;

        let background_timeout_millis = std::env::var(LB_BACKGROUND_TIMEOUT_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_BACKGROUND_TIMEOUT_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_BACKGROUND_TIMEOUT_MS_ENV_KEY, e
                ))
            })?;

        let accept_timeout_millis = std::env::var(LB_ACCEPT_TIMEOUT_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_ACCEPT_TIMEOUT_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_ACCEPT_TIMEOUT_MS_ENV_KEY, e
                ))
            })?;

        let config_watch_interval_millis =
            std::env::var(constants::LB_CONFIG_WATCH_INTERVAL_MS_ENV_KEY)
                .unwrap_or_else(|_| {
                    constants::LB_CONFIG_WATCH_INTERVAL_MS_DEFAULT.to_string()
                })
                .parse::<u64>()
                .map_err(|e| {
                    ConfigError::Parse(format!(
                        "Invalid {}: {}",
                        constants::LB_CONFIG_WATCH_INTERVAL_MS_ENV_KEY,
                        e
                    ))
                })?;

        // Proxy config
        let listen_address = std::env::var(LB_LISTEN_ADDRESS_ENV_KEY)
            .unwrap_or_else(|_| LB_LISTEN_ADDRESS_DEFAULT.to_string())
            .parse::<std::net::SocketAddr>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_LISTEN_ADDRESS_ENV_KEY, e
                ))
            })?;

        let max_connections = std::env::var(LB_MAX_CONNECTIONS_ENV_KEY)
            .ok()
            .map(|v| {
                v.parse::<u64>().map_err(|e| {
                    ConfigError::Parse(format!(
                        "Invalid {}: {}",
                        LB_MAX_CONNECTIONS_ENV_KEY, e
                    ))
                })
            })
            .transpose()?;

        // Strategy
        let strategy_str = std::env::var(LB_STRATEGY_ENV_KEY)
            .unwrap_or_else(|_| LB_STRATEGY_DEFAULT.to_string());
        let strategy = strategy_str
            .parse::<Strategy>()
            .map_err(|e: StrategyError| {
                ConfigError::Parse(format!("Invalid {}: {}", LB_STRATEGY_ENV_KEY, e))
            })?;

        // Health config
        let health_interval_ms = std::env::var(LB_HEALTH_INTERVAL_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_HEALTH_INTERVAL_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_HEALTH_INTERVAL_MS_ENV_KEY, e
                ))
            })?;

        let health_timeout_ms = std::env::var(LB_HEALTH_TIMEOUT_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_HEALTH_TIMEOUT_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_HEALTH_TIMEOUT_MS_ENV_KEY, e
                ))
            })?;

        // Metrics config
        let metrics_interval_ms = std::env::var(LB_METRICS_INTERVAL_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_METRICS_INTERVAL_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_METRICS_INTERVAL_MS_ENV_KEY, e
                ))
            })?;

        let metrics_timeout_ms = std::env::var(LB_METRICS_TIMEOUT_MS_ENV_KEY)
            .unwrap_or_else(|_| LB_METRICS_TIMEOUT_MS_DEFAULT.to_string())
            .parse::<u64>()
            .map_err(|e| {
                ConfigError::Parse(format!(
                    "Invalid {}: {}",
                    LB_METRICS_TIMEOUT_MS_ENV_KEY, e
                ))
            })?;

        let otlp_endpoint = std::env::var(LB_OTLP_ENDPOINT_ENV_KEY).ok();
        let otlp_protocol = std::env::var(LB_OTLP_PROTOCOL_ENV_KEY).ok();

        Ok(Config {
            source: ConfigSource::Environment,
            runtime: RuntimeConfig {
                metrics_cap,
                health_cap,
                drain_timeout_millis,
                background_timeout_millis,
                accept_timeout_millis,
                config_watch_interval_millis,
            },
            proxy: ProxyConfig {
                listen_address,
                max_connections,
            },
            strategy,
            backends: Vec::new(),
            health: HealthConfig {
                interval: Duration::from_millis(health_interval_ms),
                timeout: Duration::from_millis(health_timeout_ms),
            },
            metrics: MetricsConfig {
                interval: Duration::from_millis(metrics_interval_ms),
                timeout: Duration::from_millis(metrics_timeout_ms),
            },
            otlp_protocol,
            otlp_endpoint,
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
                    let mut config: Config = serde_json::from_str(&content)?;
                    config.source = ConfigSource::File;
                    Ok(config)
                }
                "toml" => {
                    let mut config: Config = toml::from_str(&content)?;
                    config.source = ConfigSource::File;
                    Ok(config)
                }
                "yaml" | "yml" => {
                    let config: Config = serde_yaml::from_str(&content)?;
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

    // Runtime config
    pub const LB_METRICS_CAP_ENV_KEY: &str = "LEMONADE_LB_METRICS_CAP";
    pub const LB_HEALTH_CAP_ENV_KEY: &str = "LEMONADE_LB_HEALTH_CAP";
    pub const LB_DRAIN_TIMEOUT_MS_ENV_KEY: &str = "LEMONADE_LB_DRAIN_TIMEOUT_MS";
    pub const LB_BACKGROUND_TIMEOUT_MS_ENV_KEY: &str =
        "LEMONADE_LB_BACKGROUND_TIMEOUT_MS";
    pub const LB_ACCEPT_TIMEOUT_MS_ENV_KEY: &str = "LEMONADE_LB_ACCEPT_TIMEOUT_MS";

    pub const LB_METRICS_CAP_DEFAULT: usize = 100;
    pub const LB_HEALTH_CAP_DEFAULT: usize = 50;
    pub const LB_DRAIN_TIMEOUT_MS_DEFAULT: u64 = 5000;
    pub const LB_BACKGROUND_TIMEOUT_MS_DEFAULT: u64 = 1000;
    pub const LB_ACCEPT_TIMEOUT_MS_DEFAULT: u64 = 2000;
    pub const LB_CONFIG_WATCH_INTERVAL_MS_ENV_KEY: &str =
        "LEMONADE_LB_CONFIG_WATCH_INTERVAL_MS";
    pub const LB_CONFIG_WATCH_INTERVAL_MS_DEFAULT: u64 = 1000;

    // Proxy config
    pub const LB_LISTEN_ADDRESS_ENV_KEY: &str = "LEMONADE_LB_LISTEN_ADDRESS";
    pub const LB_MAX_CONNECTIONS_ENV_KEY: &str = "LEMONADE_LB_MAX_CONNECTIONS";

    pub const LB_LISTEN_ADDRESS_DEFAULT: &str = "127.0.0.1:3000";
    // max_connections is optional, no default

    // Strategy
    pub const LB_STRATEGY_ENV_KEY: &str = "LEMONADE_LB_STRATEGY";
    pub const LB_STRATEGY_DEFAULT: &str = "round_robin";

    // Health config
    pub const LB_HEALTH_INTERVAL_MS_ENV_KEY: &str = "LEMONADE_LB_HEALTH_INTERVAL_MS";
    pub const LB_HEALTH_TIMEOUT_MS_ENV_KEY: &str = "LEMONADE_LB_HEALTH_TIMEOUT_MS";

    pub const LB_HEALTH_INTERVAL_MS_DEFAULT: u64 = 30000; // 10 seconds
    pub const LB_HEALTH_TIMEOUT_MS_DEFAULT: u64 = 30000; // 30 seconds

    // Metrics config
    pub const LB_METRICS_INTERVAL_MS_ENV_KEY: &str = "LEMONADE_LB_METRICS_INTERVAL_MS";
    pub const LB_METRICS_TIMEOUT_MS_ENV_KEY: &str = "LEMONADE_LB_METRICS_TIMEOUT_MS";

    pub const LB_METRICS_INTERVAL_MS_DEFAULT: u64 = 10000; // 10 seconds
    pub const LB_METRICS_TIMEOUT_MS_DEFAULT: u64 = 10000; // 10 seconds

    pub const LB_OTLP_ENDPOINT_ENV_KEY: &str = "LEMONADE_OTLP_ENDPOINT";

    pub const LB_OTLP_PROTOCOL_ENV_KEY: &str = "LEMONADE_OTLP_PROTOCOL";
}
