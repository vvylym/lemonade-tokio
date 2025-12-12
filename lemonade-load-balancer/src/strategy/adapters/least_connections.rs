use crate::prelude::*;

/// Least connections strategy implementation
#[derive(Default)]
pub struct LeastConnectionsStrategy {}

#[async_trait]
impl StrategyService for LeastConnectionsStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::LeastConnections
    }

    async fn pick_backend(
        &self,
        ctx: Arc<Context>,
    ) -> Result<BackendMeta, StrategyError> {
        let healthy = ctx.healthy_backends();

        if healthy.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Load connection registry and routing table once
        let connections = ctx.connection_registry();
        let routing = ctx.routing_table();

        // Find backend with least connections
        let backend = healthy
            .iter()
            .min_by_key(|b| {
                routing
                    .find_index(*b.id())
                    .map(|idx| connections.get(idx))
                    .unwrap_or(usize::MAX)
            })
            .ok_or(StrategyError::NoBackendAvailable)?;

        Ok(backend.clone())
    }
}
