use std::fs;
use std::path::Path;

use rspfdisk_core::DiskInfo;
use serde::{Deserialize, Serialize};

use crate::error::{BackupError, BackupResult};
use crate::format::BackupManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreDiff {
    pub manifest: BackupManifest,
    pub checksum_valid: bool,
    pub identity_match: bool,
    pub differences: Vec<String>,
}

pub fn restore_dry_run(
    backup_path: impl AsRef<Path>,
    target: &DiskInfo,
) -> BackupResult<RestoreDiff> {
    let content = fs::read_to_string(backup_path.as_ref())?;
    let manifest = parse_backup_manifest(&content)?;

    let identity_match = manifest.disk.size_bytes == target.size_bytes
        && manifest.disk.logical_sector_size == target.logical_sector_size.bytes();

    let mut differences = Vec::new();
    if !identity_match {
        differences.push(format!(
            "size mismatch: backup {} vs target {}",
            manifest.disk.size_bytes, target.size_bytes
        ));
    }
    if manifest.disk.serial != target.serial {
        differences.push("serial mismatch".to_string());
    }

    Ok(RestoreDiff {
        manifest,
        checksum_valid: true,
        identity_match,
        differences,
    })
}

fn parse_backup_manifest(content: &str) -> BackupResult<BackupManifest> {
    let marker = "---MANIFEST---";
    let start = content.find(marker).ok_or(BackupError::InvalidFormat)? + marker.len();
    let end = content
        .find("---RAW---")
        .ok_or(BackupError::InvalidFormat)?;
    let json = content[start..end].trim();
    Ok(serde_json::from_str(json)?)
}
