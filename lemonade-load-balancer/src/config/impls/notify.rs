//! Notify implementation of ConfigService
//!
//! Uses notify crate for file watching and hot-reload of configuration

use crate::config::builder::ConfigBuilder;
use crate::config::port::ConfigService;
use crate::prelude::*;
use async_trait::async_trait;
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;

/// Notify-based config service implementation
pub struct NotifyConfigService {
    /// Config file path
    config_path: Option<PathBuf>,
}

impl NotifyConfigService {
    /// Create a new NotifyConfigService
    ///
    /// # Arguments
    /// * `config_path` - Optional path to config file (JSON or TOML)
    ///
    /// # Returns
    /// * `Ok(Self)` if service was created successfully
    /// * `Err(ConfigError)` if config file doesn't exist or can't be read
    pub fn new(config_path: Option<impl Into<PathBuf>>) -> Result<Self, ConfigError> {
        let config_path = config_path.map(|p| p.into());

        // Validate config file exists if provided
        if let Some(ref path) = config_path
            && !path.exists()
        {
            return Err(ConfigError::FileNotFound(path.clone()));
        }

        Ok(Self { config_path })
    }
}

#[async_trait]
impl ConfigService for NotifyConfigService {
    #[tracing::instrument(skip(self, ctx), fields(service.name = "lemonade-load-balancer", service.type = "config"))]
    async fn watch_config(&self, ctx: Arc<Context>) {
        if let Some(ref config_path) = self.config_path {
            tracing::info!("Starting config watcher for file: {:?}", config_path);

            let mut shutdown_rx = ctx.channels().shutdown_rx();

            // Get initial watch interval from config
            let initial_config = ctx.config();
            let mut watch_interval_ms =
                initial_config.runtime.config_watch_interval_millis;

            // Create file watcher (we keep it alive to maintain watch)
            let _watcher = match notify::recommended_watcher(
                move |_result: notify::Result<notify::Event>| {
                    // File change events are handled by polling in the background task
                },
            ) {
                Ok(mut watcher) => {
                    if let Err(e) =
                        watcher.watch(config_path, RecursiveMode::NonRecursive)
                    {
                        tracing::error!("Failed to watch config file: {}", e);
                        return;
                    }
                    Some(watcher)
                }
                Err(e) => {
                    tracing::error!("Failed to create file watcher: {}", e);
                    return;
                }
            };

            // Track last modification time to only reload when file actually changes
            let mut last_mtime = std::fs::metadata(config_path)
                .and_then(|m| m.modified())
                .ok();

            // Use configurable watch interval from runtime config
            let mut debounce = tokio::time::interval(tokio::time::Duration::from_millis(
                watch_interval_ms,
            ));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Config watcher received shutdown signal");
                        break;
                    }
                    _ = debounce.tick() => {
                        // Check if watch interval changed in config and update debounce interval
                        let current_config = ctx.config();
                        let new_watch_interval_ms = current_config.runtime.config_watch_interval_millis;
                        if new_watch_interval_ms != watch_interval_ms {
                            watch_interval_ms = new_watch_interval_ms;
                            debounce = tokio::time::interval(tokio::time::Duration::from_millis(watch_interval_ms));
                            tracing::debug!("Config watch interval updated to {}ms", watch_interval_ms);
                        }

                        // Check if file was actually modified
                        let current_mtime = std::fs::metadata(config_path)
                            .and_then(|m| m.modified())
                            .ok();

                        // Only reload if modification time changed
                        if current_mtime != last_mtime {
                            last_mtime = current_mtime;

                            // Check if file was modified and reload using ConfigBuilder
                            match ConfigBuilder::from_file(Some(config_path)) {
                                Ok(new_config) => {
                                    tracing::info!("Config file changed, reloading configuration");
                                    tracing::debug!(
                                        "New config: {} backends, strategy: {:?}",
                                        new_config.backends.len(),
                                        new_config.strategy
                                    );

                                    // Call ctx.migrate() to handle all updates atomically
                                    if let Err(e) = ctx.migrate(new_config).await {
                                        tracing::error!("Failed to migrate config: {}", e);
                                    } else {
                                        tracing::debug!("Config migrated successfully");
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to reload config file, keeping previous configuration: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
            tracing::info!("Config watcher stopped");
        } else {
            tracing::info!("No config file specified, using environment variables");
            // For env vars, just wait for shutdown
            let mut shutdown_rx = ctx.channels().shutdown_rx();
            let _ = shutdown_rx.recv().await;
        }
    }
}
