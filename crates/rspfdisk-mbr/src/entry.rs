use rspfdisk_core::{PartitionEntry, PartitionType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MbrEntry {
    pub bootable: bool,
    pub partition_type: u8,
    pub start_lba: u32,
    pub sector_count: u32,
}

impl MbrEntry {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < 16 {
            return None;
        }
        if raw[0] == 0 && raw[4] == 0 {
            return None;
        }
        Some(Self {
            bootable: raw[0] == 0x80,
            partition_type: raw[4],
            start_lba: u32::from_le_bytes(raw[8..12].try_into().ok()?),
            sector_count: u32::from_le_bytes(raw[12..16].try_into().ok()?),
        })
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut raw = [0u8; 16];
        raw[0] = if self.bootable { 0x80 } else { 0x00 };
        raw[4] = self.partition_type;
        raw[8..12].copy_from_slice(&self.start_lba.to_le_bytes());
        raw[12..16].copy_from_slice(&self.sector_count.to_le_bytes());
        raw
    }

    pub fn to_partition_entry(&self, index: u32, sector_size: u32) -> PartitionEntry {
        let size_bytes = self.sector_count as u64 * sector_size as u64;
        let end_lba = self.start_lba as u64 + self.sector_count as u64 - 1;
        let partition_type = match self.partition_type {
            0xEE => PartitionType::ProtectiveMbr,
            _ => PartitionType::Unknown,
        };
        PartitionEntry {
            index,
            name: format!("MBR Partition {}", index + 1),
            start_lba: self.start_lba as u64,
            end_lba,
            size_bytes,
            partition_type,
            type_guid: None,
            partition_guid: None,
            filesystem: None,
            mount_point: None,
            bootable: self.bootable,
            active: self.bootable,
        }
    }
}
