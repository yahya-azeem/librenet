use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppManifest {
    pub name: String,
    pub version: String,
    pub compute: ComputeRequirements,
    pub storage: StorageRequirements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeRequirements {
    pub wasm_cid: String,
    pub redundancy: usize, // "Tasting the soup" factor
    pub needs_gpu: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageRequirements {
    pub total_size_mb: usize,
    pub shards: usize,
}

impl AppManifest {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
