use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::sector::SectorSize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PartitionTableKind {
    Mbr,
    Gpt,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BootMode {
    Uefi,
    Bios,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub path: String,
    pub size_bytes: u64,
    pub logical_sector_size: SectorSize,
    pub physical_sector_size: Option<SectorSize>,
    pub model: Option<String>,
    pub serial: Option<String>,
    pub removable: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PartitionType {
    Esp,
    Msr,
    MicrosoftBasicData,
    WindowsRecovery,
    LinuxFilesystem,
    LinuxSwap,
    AppleApfs,
    BiosBoot,
    ProtectiveMbr,
    Unknown,
    Custom(String),
}

impl PartitionType {
    pub fn from_slug(slug: &str) -> Self {
        match slug {
            "esp" => Self::Esp,
            "msr" => Self::Msr,
            "microsoft-basic-data" => Self::MicrosoftBasicData,
            "windows-recovery" => Self::WindowsRecovery,
            "linux-filesystem" => Self::LinuxFilesystem,
            "linux-swap" => Self::LinuxSwap,
            "apple-apfs" => Self::AppleApfs,
            "bios-boot" => Self::BiosBoot,
            other => Self::Custom(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionEntry {
    pub index: u32,
    pub name: String,
    pub start_lba: u64,
    pub end_lba: u64,
    pub size_bytes: u64,
    pub partition_type: PartitionType,
    pub type_guid: Option<Uuid>,
    pub partition_guid: Option<Uuid>,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub bootable: bool,
    pub active: bool,
}

impl PartitionEntry {
    pub fn sector_count(&self, sector_size: SectorSize) -> u64 {
        self.size_bytes / sector_size.bytes() as u64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionTable {
    pub kind: PartitionTableKind,
    pub boot_mode: BootMode,
    pub partitions: Vec<PartitionEntry>,
    pub disk_guid: Option<Uuid>,
    pub warnings: Vec<String>,
}

impl PartitionTable {
    pub fn empty() -> Self {
        Self {
            kind: PartitionTableKind::Unknown,
            boot_mode: BootMode::Unknown,
            partitions: Vec::new(),
            disk_guid: None,
            warnings: Vec::new(),
        }
    }
}
