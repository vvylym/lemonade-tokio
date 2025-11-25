use std::sync::Arc;

use crate::{algorithms::Algorithm, backend::BackendId, config::Config, error::Result, strategies::*};

pub struct AppState {
    // TODO: describe fields here
}

impl AppState {
    /// Constructor
    pub fn new(config: &Config) -> Self {
        // TODO: finalize the creation of the state from the config
        todo!()
    }
}

impl AppState {
    /// 
    fn select_algorithm(&self) -> Algorithm {
        todo!()
    }

    /// Select the best backend
    pub async fn select_best_backend(&self) -> Result<BackendId> {
        // TODO: to be updated with the various appropriate arguments
        
        match self.select_algorithm() {
            Algorithm::Adaptive => adaptive::execute_strategy().await,
            Algorithm::RoundRobin => round_robin::execute_strategy().await,
            Algorithm::LeastConnections => least_connections::execute_strategy().await,
            Algorithm::WeightedRoundRobin => weighted_round_robin::execute_strategy().await,
        }
    }
}
