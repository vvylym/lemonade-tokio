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
        let routing = ctx.routing_table();
        let healthy = routing.healthy_backends();

        if healthy.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Find backend with least connections
        let backend = healthy
            .iter()
            .min_by_key(|b| b.active_connections())
            .ok_or(StrategyError::NoBackendAvailable)?;

        Ok(BackendMeta::new(
            backend.id(),
            backend.name(),
            backend.address(),
            backend.weight(),
        ))
    }
}
