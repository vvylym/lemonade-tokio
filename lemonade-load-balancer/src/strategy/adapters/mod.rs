//! Strategy implementations module
//!

mod adaptive;
mod fastest_response_time;
mod least_connections;
mod round_robin;
mod weighted_round_robin;

pub use adaptive::*;
pub use fastest_response_time::*;
pub use least_connections::*;
pub use round_robin::*;
pub use weighted_round_robin::*;
