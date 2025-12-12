//! Work module
//!

mod error;
mod models;
mod port;

pub use error::WorkError;
pub use models::WorkResponse;
pub use port::WorkService;
