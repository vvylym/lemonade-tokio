//! Context module
//!
//! Shared context for load balancer services with encapsulation

use crate::prelude::*;
pub use error::ContextError;
use std::sync::Mutex;
use tokio::sync::Notify;

/// App context struct - all fields private for encapsulation
pub struct Context {
    // All fields private
    config: ArcSwap<Config>,
    route_table: ArcSwap<RouteTable>,
    strategy: ArcSwap<Arc<dyn StrategyService>>,
    channels: Arc<ChannelBundle>,
    migration_lock: Mutex<()>,
    // Notify for connection drain waiting
    connection_notify: Arc<Notify>,
}

impl Context {
    /// Create a new context from config
    pub fn new(config: Config) -> Result<Self, ContextError> {
        // Create channel bundle
        let channels = Arc::new(ChannelBundle::new(
            config.runtime.metrics_cap,
            config.runtime.health_cap,
            config.runtime.metrics_cap, // Use metrics_cap for connection_cap
            100,                        // backend_failure_cap
        ));

        // Create route table from backend configs
        let route_table = ArcSwap::from_pointee(RouteTable::new(config.backends.clone()));

        // Build strategy (convert BackendConfig to BackendMeta for compatibility)
        let backend_metas: Vec<BackendMeta> = config
            .backends
            .iter()
            .map(|c| BackendMeta::new(c.id, c.name.clone(), c.address.clone(), c.weight))
            .collect();
        let strategy = StrategyBuilder::new()
            .with_strategy(config.strategy.clone())
            .with_backends(backend_metas)
            .build()?;

        Ok(Self {
            config: ArcSwap::from_pointee(config),
            route_table,
            strategy: ArcSwap::from_pointee(strategy),
            channels,
            migration_lock: Mutex::new(()),
            connection_notify: Arc::new(Notify::new()),
        })
    }

    // Getters (no direct field access)

    /// Get config
    pub fn config(&self) -> Arc<Config> {
        self.config.load_full()
    }

    /// Get routing table
    pub fn routing_table(&self) -> Arc<RouteTable> {
        self.route_table.load_full()
    }

    /// Get strategy
    pub fn strategy(&self) -> Arc<Arc<dyn StrategyService>> {
        self.strategy.load_full()
    }

    /// Get channels
    pub fn channels(&self) -> &ChannelBundle {
        &self.channels
    }

    // Private setters (used internally by migrate)

    fn set_config(&self, config: Arc<Config>) {
        self.config.store(config);
    }

    fn set_strategy(&self, strategy: Arc<dyn StrategyService>) {
        self.strategy.store(Arc::new(strategy));
    }

    fn set_routing_table(&self, rt: Arc<RouteTable>) {
        self.route_table.store(rt);
    }

    /// Migrate to new config (handles backend draining, config/strategy update, listen address change)
    pub async fn migrate(&self, new_config: Config) -> Result<(), ContextError> {
        // Acquire migration lock for critical section
        let _lock = self.migration_lock.lock().unwrap();

        let old_config = self.config();
        let old_routing = self.routing_table();

        // Check if listen address changed
        if old_config.proxy.listen_address != new_config.proxy.listen_address {
            let _ = self
                .channels
                .config_tx()
                .send(ConfigEvent::ListenAddressChanged(
                    new_config.proxy.listen_address,
                ));
        }

        // Compare old vs new backends
        let old_backends: std::collections::HashMap<BackendId, Arc<Backend>> =
            old_routing
                .all_backends()
                .into_iter()
                .map(|b| (b.id(), b))
                .collect();

        let new_backend_configs: std::collections::HashMap<BackendId, BackendConfig> =
            new_config
                .backends
                .iter()
                .map(|c| (c.id, c.clone()))
                .collect();

        // Identify backends to drain (removed or changed)
        let mut to_drain: Vec<Arc<Backend>> = Vec::new();
        let mut to_add: Vec<BackendConfig> = Vec::new();

        // Check for removed or changed backends
        for (id, old_backend) in &old_backends {
            if let Some(new_config) = new_backend_configs.get(id) {
                // Backend exists - check if changed
                let old_addr = old_backend.address();
                let old_name = old_backend.name();
                if new_config.address != *old_addr
                    || new_config.name.as_deref() != old_name
                    || new_config.weight != old_backend.weight()
                {
                    // Backend changed - mark old as draining
                    to_drain.push(old_backend.clone());
                    to_add.push(new_config.clone());
                }
            } else {
                // Backend removed - mark as draining
                to_drain.push(old_backend.clone());
            }
        }

        // Check for new backends
        for (id, new_config) in &new_backend_configs {
            if !old_backends.contains_key(id) {
                to_add.push(new_config.clone());
            }
        }

        // Mark backends as draining
        for backend in &to_drain {
            backend.mark_draining();
        }

        // Create new route table
        let mut new_backends: Vec<Arc<Backend>> = old_routing
            .all_backends()
            .into_iter()
            .filter(|b| !to_drain.iter().any(|d| d.id() == b.id()))
            .collect();

        // Add new backends
        for config in &to_add {
            new_backends.push(Arc::new(Backend::new(config.clone())));
        }

        // Create new route table
        let new_route_table = RouteTable::default();
        for backend in new_backends {
            new_route_table.insert(backend);
        }

        // Prepare strategy update (convert BackendConfig to BackendMeta)
        let backend_metas: Vec<BackendMeta> = new_config
            .backends
            .iter()
            .map(|c| BackendMeta::new(c.id, c.name.clone(), c.address.clone(), c.weight))
            .collect();
        let new_strategy = StrategyBuilder::new()
            .with_strategy(new_config.strategy.clone())
            .with_backends(backend_metas)
            .build()?;

        // Release lock before await (waiting for drain)
        drop(_lock);

        // Wait for draining backends to have 0 connections (with timeout)
        let drain_timeout =
            Duration::from_millis(new_config.runtime.drain_timeout_millis);
        let start = std::time::Instant::now();

        while start.elapsed() < drain_timeout {
            let all_drained = to_drain
                .iter()
                .all(|backend| backend.active_connections() == 0);

            if all_drained {
                break;
            }

            // Wait a bit before checking again
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Re-acquire lock for final updates
        let _lock2 = self.migration_lock.lock().unwrap();

        // Update config, strategy, route table atomically
        self.set_config(Arc::new(new_config.clone()));
        self.set_strategy(new_strategy);
        self.set_routing_table(Arc::new(new_route_table));

        // Broadcast ConfigEvent::Migrated
        let _ = self.channels.config_tx().send(ConfigEvent::Migrated);

        Ok(())
    }

    /// Wait for all connections to drain (for shutdown)
    pub async fn wait_for_drain(&self, timeout: Duration) -> Result<(), ContextError> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            let routing = self.routing_table();
            let total_connections: usize = routing
                .all_backends()
                .iter()
                .map(|b| b.active_connections())
                .sum();

            if total_connections == 0 {
                return Ok(());
            }

            // Wait for notification or timeout
            let remaining = timeout - start.elapsed();
            if remaining.is_zero() {
                break;
            }

            tokio::select! {
                _ = self.connection_notify.notified() => {
                    // Connection closed, check again
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Timeout, check again
                }
            }
        }

        // Check one more time
        let routing = self.routing_table();
        let total_connections: usize = routing
            .all_backends()
            .iter()
            .map(|b| b.active_connections())
            .sum();

        if total_connections > 0 {
            return Err(ContextError::DrainTimeout(format!(
                "{} connections still active after timeout",
                total_connections
            )));
        }

        Ok(())
    }

    /// Notify that a connection was closed (for drain waiting)
    pub fn notify_connection_closed(&self) {
        self.connection_notify.notify_one();
    }
}

mod error {
    //! Error module
    //!
    use crate::prelude::*;

    /// Context error enum
    #[derive(Debug, thiserror::Error)]
    pub enum ContextError {
        /// Strategy builder error
        #[error("strategy builder error: {0}")]
        StrategyBuilder(#[from] StrategyError),
        /// Drain timeout error
        #[error("drain timeout: {0}")]
        DrainTimeout(String),
    }
}
