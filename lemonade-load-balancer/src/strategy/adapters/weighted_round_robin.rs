use crate::prelude::*;

/// Weighted round robin strategy implementation
#[derive(Default)]
pub struct WeightedRoundRobinStrategy {
    current_index: AtomicUsize,
}

#[async_trait]
impl StrategyService for WeightedRoundRobinStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::WeightedRoundRobin
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

        // Build weights vector for current healthy backends
        let weights: Vec<usize> = healthy
            .iter()
            .map(|b| b.weight().unwrap_or(1) as usize)
            .collect();

        let total_weight: usize = weights.iter().sum();

        if total_weight == 0 {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Weighted round robin selection
        let current = self.current_index.fetch_add(1, Ordering::Relaxed);
        let mut weight_sum = 0;
        let target = current % total_weight;

        for (i, backend) in healthy.iter().enumerate() {
            weight_sum += weights[i];
            if target < weight_sum {
                return Ok(BackendMeta::new(
                    backend.id(),
                    backend.name(),
                    backend.address(),
                    backend.weight(),
                ));
            }
        }

        // Fallback to first backend (should never reach here)
        let backend = &healthy[0];
        Ok(BackendMeta::new(
            backend.id(),
            backend.name(),
            backend.address(),
            backend.weight(),
        ))
    }
}
