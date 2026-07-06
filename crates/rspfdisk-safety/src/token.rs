use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteToken {
    pub disk_id: String,
    pub timestamp: DateTime<Utc>,
    pub backup_id: Option<String>,
    pub confirmation_hash: String,
    pub change_plan_hash: String,
}

impl WriteToken {
    pub fn new(
        disk_id: String,
        backup_id: Option<String>,
        confirmation_phrase: &str,
        change_plan_json: &str,
    ) -> Self {
        let confirmation_hash = hash_str(confirmation_phrase);
        let change_plan_hash = hash_str(change_plan_json);
        Self {
            disk_id,
            timestamp: Utc::now(),
            backup_id,
            confirmation_hash,
            change_plan_hash,
        }
    }
}

fn hash_str(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}
