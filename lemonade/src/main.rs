//! Lemonade CLI
//!

/// Main entrypoint
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lemonade::run().await
}
