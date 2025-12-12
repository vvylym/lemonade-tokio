//! Serde helpers for config serialization
//!
use serde::{Deserialize, Deserializer, Serializer};
use std::time::Duration;

/// Serialize Duration as milliseconds (u64)
pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u64(duration.as_millis() as u64)
}

/// Deserialize Duration from milliseconds (u64)
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let ms = u64::deserialize(deserializer)?;
    Ok(Duration::from_millis(ms))
}

