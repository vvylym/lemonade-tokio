//! Backend meta module
//!
use crate::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Backend meta struct
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendMeta {
    /// Unique identifier for the backend
    id: BackendId,
    /// Name of the backend
    name: Option<String>,
    /// Socket address for the backend
    address: BackendAddress,
    /// Weight of the backend
    weight: Option<u8>,
}

#[derive(Serialize, Deserialize)]
struct BackendMetaSerde {
    id: BackendId,
    name: Option<String>,
    address: BackendAddress,
    weight: Option<u8>,
}

impl Serialize for BackendMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        BackendMetaSerde {
            id: self.id,
            name: self.name.clone(),
            address: self.address.clone(),
            weight: self.weight,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BackendMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let serde = BackendMetaSerde::deserialize(deserializer)?;
        Ok(BackendMeta {
            id: serde.id,
            name: serde.name,
            address: serde.address,
            weight: serde.weight,
        })
    }
}

impl BackendMeta {
    /// Create a new backend meta
    pub fn new(
        id: impl Into<BackendId>,
        name: Option<impl Into<String>>,
        address: impl Into<BackendAddress>,
        weight: Option<impl Into<u8>>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.map(|n| n.into()),
            address: address.into(),
            weight: weight.map(|w| w.into()),
        }
    }

    /// Get the backend id
    pub fn id(&self) -> &BackendId {
        &self.id
    }

    /// Get the backend name
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Get the backend address
    pub fn address(&self) -> &BackendAddress {
        &self.address
    }

    /// Get the backend weight
    pub fn weight(&self) -> Option<u8> {
        self.weight
    }
}
