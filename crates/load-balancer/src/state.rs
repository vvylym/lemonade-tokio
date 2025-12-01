use crate::config::Config;

/// Application state
#[derive(Debug, Clone)]
pub struct AppState {
    // TODO: describe fields here
}

impl AppState {
    /// Constructor
    pub fn new(_config: &Config) -> Self {
        todo!("finalize the creation of the state from the config")
    }
}
