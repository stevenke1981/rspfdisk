use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDiskInfo {
    pub path: String,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub size_bytes: u64,
    pub logical_sector_size: u32,
    pub physical_sector_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub format: String,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub tool: String,
    pub disk: BackupDiskInfo,
    pub partition_table: String,
    pub notes: Vec<String>,
}

impl BackupManifest {
    pub fn new(disk: BackupDiskInfo, partition_table: &str) -> Self {
        Self {
            format: "rspbak".to_string(),
            version: 1,
            created_at: Utc::now(),
            tool: "rspfdisk 0.1.0".to_string(),
            disk,
            partition_table: partition_table.to_string(),
            notes: Vec::new(),
        }
    }
}
