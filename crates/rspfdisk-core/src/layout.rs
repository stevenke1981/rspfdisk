use serde::{Deserialize, Serialize};

use crate::partition::{BootMode, PartitionTableKind, PartitionType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionDraft {
    pub name: String,
    pub start_lba: u64,
    pub size_bytes: u64,
    pub partition_type: PartitionType,
    pub filesystem: Option<String>,
    pub mount_point: Option<String>,
    pub note: Option<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDraft {
    pub template_name: String,
    pub display_name: String,
    pub table: PartitionTableKind,
    pub boot_mode: BootMode,
    pub partitions: Vec<PartitionDraft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub creates_gpt: bool,
    pub creates_mbr: bool,
    pub added_partitions: Vec<String>,
    pub removed_partitions: Vec<String>,
    pub modified_partitions: Vec<String>,
    pub summary_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePlan {
    pub disk_path: String,
    pub layout: LayoutDraft,
    pub diff: DiffReport,
    pub backup_path: Option<String>,
    pub dry_run: bool,
}
