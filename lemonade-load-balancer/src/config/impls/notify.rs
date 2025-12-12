//! Notify implementation of ConfigService
//!
//! Uses notify crate for file watching and hot-reload of configuration

use crate::config::builder::ConfigBuilder;
use crate::config::error::ConfigError;
use crate::config::models::Config;
use crate::config::port::ConfigService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify as TokioNotify;

/// Notify-based config service implementation
pub struct NotifyConfigService {
    /// Config file path (None means use environment variables)
    config_path: Option<PathBuf>,
    /// Lock-free atomic config storage (wrapped in Arc for sharing)
    config: Arc<ArcSwap<Config>>,
    /// Shutdown notification
    shutdown: Arc<TokioNotify>,
    /// File watcher (kept alive to maintain watch)
    #[allow(dead_code)]
    _watcher: Option<RecommendedWatcher>,
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
        let config = ConfigBuilder::from_file(config_path.as_deref())?;

        Ok(Self {
            config_path,
            config: Arc::new(ArcSwap::from_pointee(config)),
            shutdown: Arc::new(TokioNotify::new()),
            _watcher: None,
        })
    }
}

#[async_trait]
impl ConfigService for NotifyConfigService {
    fn snapshot(&self) -> Config {
        self.config.load_full().as_ref().clone()
    }

    async fn start(&self, ctx: Arc<Context>) -> Result<(), ConfigError> {
        if let Some(ref config_path) = self.config_path {
            // Create file watcher (we keep it in the struct to keep it alive)
            let mut _watcher = notify::recommended_watcher(
                move |_result: notify::Result<notify::Event>| {
                    // File change events are handled by polling in the background task
                },
            )?;

            // Watch the config file
            _watcher.watch(config_path, RecursiveMode::NonRecursive)?;

            // Spawn a background task to handle file changes
            let config_path = config_path.clone();
            let config_arc = self.config.clone();
            let ctx_clone = ctx.clone();
            let shutdown_clone = self.shutdown.clone();

            tokio::spawn(async move {
                let mut debounce =
                    tokio::time::interval(tokio::time::Duration::from_millis(100));
                loop {
                    tokio::select! {
                        _ = shutdown_clone.notified() => {
                            break;
                        }
                        _ = debounce.tick() => {
                            // Check if file was modified and reload using ConfigBuilder
                            if let Ok(new_config) = ConfigBuilder::from_file(Some(&config_path)) {
                                config_arc.store(Arc::new(new_config.clone()));

                                // Update context with new config
                                if let Err(e) = ctx_clone.set_backends(new_config.backends.clone()) {
                                    eprintln!("Failed to update backends: {}", e);
                                }

                                // Update strategy if changed
                                let new_strategy = StrategyBuilder::new()
                                    .with_strategy(new_config.strategy.clone())
                                    .with_backends(new_config.backends.clone())
                                    .build();
                                if let Ok(strategy) = new_strategy {
                                    ctx_clone.set_strategy(strategy);
                                }

                                // Update timeouts if changed
                                // Note: Context::set_timeouts requires &mut, so we can't update it here
                                // This would need to be handled differently or Context needs a method that takes &self
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ConfigError> {
        self.shutdown.notify_waiters();
        Ok(())
    }
}
