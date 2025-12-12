//! State module
//!
use crate::prelude::*;
pub use error::ContextError;

/// App state struct
pub struct Context {
    /// Routing table
    route_table: ArcSwap<RouteTable>,
    /// Dynamic Strategy
    strategy: ArcSwap<Arc<dyn StrategyService>>,
    /// Connection registry
    connections: ArcSwap<ConnectionRegistry>,
    /// Health registry
    health: ArcSwap<HealthRegistry>,
    /// Metrics registry
    metrics: ArcSwap<MetricsSnapshot>,
    /// Channels
    channels: ArcSwap<ChannelBundle>,
    /// Channel version sender
    channel_version_tx: watch::Sender<u64>,
    /// Notify
    notify: Notify,
    /// Drain timeout duration
    drain_timeout_duration: Duration,
    /// Background handle timeout duration
    background_handle_timeout: Duration,
    /// Accept handle timeout duration
    accept_handle_timeout: Duration,
    /// Metrics receiver
    metrics_rx: ArcSwap<MpscReceiver<MetricsEvent>>,
    /// Health receiver
    health_rx: ArcSwap<MpscReceiver<HealthEvent>>,
    /// Shutdown receiver
    shutdown_tx: BroadcastSender<()>,
}

impl Context {
    /// Create a new state
    pub fn new(config: &Config) -> Result<Self, ContextError> {
        // create channel bundle from config
        let (bundle, metrics_rx, health_rx) =
            ChannelBundle::new(config.runtime.metrics_cap, config.runtime.health_cap);
        // create channel version sender
        let (tx, _) = watch::channel(0u64);
        // create shutdown channel
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        let strategy = StrategyBuilder::new()
            .with_strategy(config.strategy.clone())
            .with_backends(config.backends.clone())
            .build()?;

        // return state
        Ok(Self {
            route_table: ArcSwap::from_pointee(RouteTable::new(config.backends.clone())),
            strategy: ArcSwap::from_pointee(strategy),
            metrics: ArcSwap::from_pointee(MetricsSnapshot::default()),
            connections: ArcSwap::from_pointee(ConnectionRegistry::new(0)),
            health: ArcSwap::from_pointee(HealthRegistry::new(0)),
            channels: ArcSwap::from_pointee(bundle),
            channel_version_tx: tx,
            notify: Notify::new(),
            drain_timeout_duration: Duration::from_millis(
                config.runtime.drain_timeout_millis,
            ),
            background_handle_timeout: Duration::from_millis(
                config.runtime.background_timeout_millis,
            ),
            accept_handle_timeout: Duration::from_millis(
                config.runtime.accept_timeout_millis,
            ),
            metrics_rx: ArcSwap::from_pointee(metrics_rx),
            health_rx: ArcSwap::from_pointee(health_rx),
            shutdown_tx,
        })
    }

    /// Get healthy backends (for strategies)
    /// Returns Vec<BackendMeta> - cloned backend metadata
    pub fn healthy_backends(&self) -> Vec<BackendMeta> {
        let routing = self.routing_table();
        let health = self.health_registry();

        routing
            .filter_healthy(&health)
            .into_iter()
            .map(|(_, meta)| meta.clone())
            .collect()
    }

    //========================================================================//
    //
    // Loaders (these are necessary for services to access attributes)
    //
    //========================================================================//

    /// Load strategy
    pub fn strategy(&self) -> Arc<Arc<dyn StrategyService>> {
        self.strategy.load_full()
    }

    /// Load routing table
    pub fn routing_table(&self) -> Arc<RouteTable> {
        self.route_table.load_full()
    }

    /// Get channel bundle (for services to send events)
    pub fn channel_bundle(&self) -> Arc<ChannelBundle> {
        self.channels.load_full()
    }

    /// Load routing table
    pub fn connection_registry(&self) -> Arc<ConnectionRegistry> {
        self.connections.load_full()
    }

    /// Load health registry
    pub fn health_registry(&self) -> Arc<HealthRegistry> {
        self.health.load_full()
    }

    /// Load metrics snapshot
    pub fn metrics_snapshot(&self) -> Arc<MetricsSnapshot> {
        self.metrics.load_full()
    }

    //========================================================================//
    //
    // Swappers (for config updates)
    //
    //========================================================================//

    /// Set channel bundle (for config service)
    pub fn set_channel_bundle(
        &self,
        new_bundle: Arc<ChannelBundle>,
        new_metrics_rx: MpscReceiver<MetricsEvent>,
        new_health_rx: MpscReceiver<HealthEvent>,
    ) {
        let old = self.channels.swap(new_bundle);
        self.metrics_rx.store(Arc::new(new_metrics_rx));
        self.health_rx.store(Arc::new(new_health_rx));
        let _ = self
            .channel_version_tx
            .send(self.channel_version_tx.borrow().wrapping_add(1));
        drop(old);
    }

    /// Update timeouts (for config service)
    pub fn set_timeouts(
        &mut self,
        drain: Option<Duration>,
        background: Option<Duration>,
        accept: Option<Duration>,
    ) {
        if let Some(d) = drain {
            self.drain_timeout_duration = d;
        }
        if let Some(b) = background {
            self.background_handle_timeout = b;
        }
        if let Some(a) = accept {
            self.accept_handle_timeout = a;
        }
    }

    /// Swap routing table
    pub fn set_routing_table(&self, rt: Arc<RouteTable>) {
        self.route_table.store(rt);
    }

    /// Swap connection registry
    pub fn set_connection_registry(&self, cr: Arc<ConnectionRegistry>) {
        self.connections.store(cr);
    }

    /// Swap health registry
    pub fn set_health_registry(&self, hr: Arc<HealthRegistry>) {
        self.health.store(hr);
    }

    /// Swap metrics snapshot
    pub fn set_metrics_snapshot(&self, ms: Arc<MetricsSnapshot>) {
        self.metrics.store(ms);
    }

    /// Swap strategy
    pub fn set_strategy(&self, strategy: Arc<dyn StrategyService>) {
        self.strategy.store(Arc::new(strategy));
    }

    /// Update backends while preserving connection counts
    pub fn set_backends(&self, backends: Vec<BackendMeta>) -> Result<(), ContextError> {
        let old_routing = self.routing_table();
        let old_connections = self.connection_registry();
        let old_health = self.health_registry();

        // Calculate new capacity
        let max_id = backends.iter().map(|b| *b.id() as usize).max().unwrap_or(0);
        let new_cap = max_id + 1;
        let old_cap = old_routing.len();

        // Build ID mapping: old index -> new index
        let id_mapping: Vec<Option<usize>> = (0..old_cap)
            .map(|old_idx| {
                old_routing.get_by_index(old_idx).and_then(|b| {
                    backends.iter().position(|new_b| *new_b.id() == *b.id())
                })
            })
            .collect();

        // Migrate connection counts
        let new_connections = old_connections.migrate(new_cap, &id_mapping);

        // Migrate health status
        let new_health = old_health.migrate(new_cap, &id_mapping);

        // Create new route table
        let new_routing = RouteTable::new(backends);

        // Update strategy if needed (check if strategy or weights changed)
        // This would be done by config service after calling update_backends

        // Swap all registries atomically
        self.route_table.store(Arc::new(new_routing));
        self.connections.store(Arc::new(new_connections));
        self.health.store(Arc::new(new_health));

        Ok(())
    }

    //========================================================================//
    //
    // Channels
    //
    //========================================================================//

    /// Get metrics receiver (services take ownership when needed)
    pub fn metrics_receiver(&self) -> Arc<MpscReceiver<MetricsEvent>> {
        self.metrics_rx.load_full().clone()
    }

    /// Get health receiver (services take ownership when needed)
    pub fn health_receiver(&self) -> Arc<MpscReceiver<HealthEvent>> {
        self.health_rx.load_full().clone()
    }

    /// keep_alive awaits notification with a small sleep to avoid busy loops
    pub async fn keep_alive(&self) -> Result<(), std::io::Error> {
        // notify is signaled by connection handlers on disconnect; we wait with timeout
        self.notify.notified().await;
        Ok(())
    }

    /// Notify connection closed (for drain logic - uses notify)
    pub fn notify_connection_closed(&self) {
        self.notify.notify_one();
    }

    /// Get channel version receiver (for config service to notify consumers)
    pub fn channel_version_receiver(&self) -> watch::Receiver<u64> {
        self.channel_version_tx.subscribe()
    }

    /// Get shutdown sender (for config service to notify shutdown)
    pub fn shutdown_sender(&self) -> BroadcastSender<()> {
        self.shutdown_tx.clone()
    }

    /// Get shutdown receiver (for config service to notify shutdown)
    pub fn shutdown_receiver(&self) -> BroadcastReceiver<()> {
        self.shutdown_tx.subscribe()
    }

    //========================================================================//
    //
    // Timeout getters
    //
    //========================================================================//

    /// Get drain timeout duration
    pub fn drain_timeout(&self) -> Duration {
        self.drain_timeout_duration
    }

    /// Get background handle timeout duration
    pub fn background_handle_timeout(&self) -> Duration {
        self.background_handle_timeout
    }

    /// Get accept handle timeout duration
    pub fn accept_handle_timeout(&self) -> Duration {
        self.accept_handle_timeout
    }
}

mod error {
    //! Error module
    //!
    use crate::prelude::*;

    /// State error enum
    #[derive(Debug, thiserror::Error)]
    pub enum ContextError {
        /// Strategy builder error
        #[error("strategy builder error: {0}")]
        StrategyBuilder(#[from] StrategyError),
    }
}
