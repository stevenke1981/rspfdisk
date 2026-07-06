use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use rspfdisk_disk::BlockDevice;
use rspfdisk_gpt::parse_gpt;
use sha2::{Digest, Sha256};

use crate::error::BackupResult;
use crate::format::{BackupDiskInfo, BackupManifest};

const RAW_SECTOR_COUNT: u64 = 34;

pub fn create_backup<D: BlockDevice>(
    device: &D,
    out_path: impl AsRef<Path>,
) -> BackupResult<BackupManifest> {
    let info = device.info();
    let partition_table = match parse_gpt(device) {
        Ok(_) => "gpt",
        Err(_) => "unknown",
    };

    let manifest = BackupManifest::new(
        BackupDiskInfo {
            path: info.path.clone(),
            model: info.model.clone(),
            serial: info.serial.clone(),
            size_bytes: info.size_bytes,
            logical_sector_size: info.logical_sector_size.bytes(),
            physical_sector_size: info.physical_sector_size.map(|s| s.bytes()),
        },
        partition_table,
    );

    let out_path = out_path.as_ref();
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let raw = device.read_sectors(0, RAW_SECTOR_COUNT.min(device.sector_count()))?;
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    let checksum = sha256_hex(raw.as_bytes());

    let mut file = File::create(out_path)?;
    writeln!(file, "RSPBAK1")?;
    writeln!(file, "---MANIFEST---")?;
    write!(file, "{manifest_json}")?;
    writeln!(file)?;
    writeln!(file, "---RAW---")?;
    write!(file, "{checksum}")?;
    writeln!(file)?;
    file.write_all(raw.as_bytes())?;

    Ok(manifest)
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
