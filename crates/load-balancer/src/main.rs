//! Load Balancer main module
//!
//! This module contains the main function for the load balancer.

/// Main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run the load balancer (hidding all the plumbing details)
    load_balancer::run().await?;

    Ok(())
}
