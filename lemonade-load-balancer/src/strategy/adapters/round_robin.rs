use crate::prelude::*;

/// Round robin strategy implementation
#[derive(Default)]
pub struct RoundRobinStrategy {
    counter: AtomicUsize,
}

#[async_trait]
impl StrategyService for RoundRobinStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::RoundRobin
    }

    async fn pick_backend(
        &self,
        ctx: Arc<Context>,
    ) -> Result<BackendMeta, StrategyError> {
        let healthy = ctx.healthy_backends();

        if healthy.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Round robin over healthy backends
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % healthy.len();
        Ok(healthy[idx].clone())
    }
}
