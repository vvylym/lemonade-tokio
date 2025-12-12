//! Prelude module
//!

// Re-export internal types for convenience
pub use crate::{
    // Config module
    config::{builder::*, error::*, impls::*, models::*, port::*},
    // Health module
    health::{adapters::*, error::*, models::*, port::*},
    // Metrics module
    metrics::{adapters::*, error::*, models::*, port::*},
    // Proxy module
    proxy::{adapters::*, error::*, models::*, port::*},
    // Strategy module
    strategy::{adapters::*, builder::*, constants::*, error::*, models::*, port::*},
    // Common types module
    types::*,
};

// Re-used types for convenience

// Async traits
pub use async_trait::async_trait;

// SocketAddr for network addresses
pub use std::net::SocketAddr;
// Arc for shared references
pub use std::sync::Arc;
// Atomic types
pub use std::sync::atomic::{AtomicU8, AtomicU64, AtomicUsize, Ordering};
// Duration for time intervals
pub use std::time::Duration;

// DashMap for concurrent hash maps
pub use dashmap::DashMap;
// ArcSwap for atomic shared references
pub use arc_swap::ArcSwap;

// Tokio modules
pub use tokio::{
    // Network types
    // net::{TcpListener, TcpStream},
    // Channels
    sync::{
        // Notify for event notifications
        Notify,
        // Broadcast channels
        broadcast::{self, Receiver as BroadcastReceiver, Sender as BroadcastSender},
        // Message channels
        mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender},
        // Watch channels
        watch::{self, Sender as WatchSender},
    },
    // Timeouts
    time::timeout,
};
