use rspfdisk_core::{
    BootMode, PartitionEntry, PartitionTable, PartitionTableKind, PartitionType, SectorSize,
};
use rspfdisk_disk::BlockDevice;
use uuid::Uuid;

use crate::crc::gpt_crc32;
use crate::error::{GptError, GptResult};
use crate::guid::{decode_utf16le_name, partition_type_guid};
use crate::header::GptHeader;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct GptPartitionEntry {
    pub type_guid: Uuid,
    pub partition_guid: Uuid,
    pub first_lba: u64,
    pub last_lba: u64,
    pub attributes: u64,
    pub name: String,
}

impl GptPartitionEntry {
    pub fn parse(raw: &[u8]) -> Option<Self> {
        if raw.len() < GPT_ENTRY_SIZE {
            return None;
        }
        let type_guid = Uuid::from_bytes(raw[0..16].try_into().ok()?);
        if type_guid == Uuid::nil() {
            return None;
        }
        let first_lba = u64::from_le_bytes(raw[32..40].try_into().ok()?);
        let last_lba = u64::from_le_bytes(raw[40..48].try_into().ok()?);
        if first_lba == 0 && last_lba == 0 {
            return None;
        }
        Some(Self {
            type_guid,
            partition_guid: Uuid::from_bytes(raw[16..32].try_into().ok()?),
            first_lba,
            last_lba,
            attributes: u64::from_le_bytes(raw[48..56].try_into().ok()?),
            name: decode_utf16le_name(&raw[56..56 + GPT_PARTITION_NAME_BYTES]),
        })
    }

    pub fn to_partition_entry(&self, index: u32, sector_size: SectorSize) -> PartitionEntry {
        let size_bytes = (self.last_lba - self.first_lba + 1) * sector_size.bytes() as u64;
        PartitionEntry {
            index,
            name: self.name.clone(),
            start_lba: self.first_lba,
            end_lba: self.last_lba,
            size_bytes,
            partition_type: partition_type_from_guid(&self.type_guid),
            type_guid: Some(self.type_guid),
            partition_guid: Some(self.partition_guid),
            filesystem: None,
            mount_point: None,
            bootable: self.attributes & 0x1 != 0,
            active: false,
        }
    }
}

fn partition_type_from_guid(guid: &Uuid) -> PartitionType {
    let known = [
        (partition_type_guid(&PartitionType::Esp), PartitionType::Esp),
        (partition_type_guid(&PartitionType::Msr), PartitionType::Msr),
        (
            partition_type_guid(&PartitionType::MicrosoftBasicData),
            PartitionType::MicrosoftBasicData,
        ),
        (
            partition_type_guid(&PartitionType::WindowsRecovery),
            PartitionType::WindowsRecovery,
        ),
        (
            partition_type_guid(&PartitionType::LinuxFilesystem),
            PartitionType::LinuxFilesystem,
        ),
        (
            partition_type_guid(&PartitionType::LinuxSwap),
            PartitionType::LinuxSwap,
        ),
        (
            partition_type_guid(&PartitionType::AppleApfs),
            PartitionType::AppleApfs,
        ),
        (
            partition_type_guid(&PartitionType::BiosBoot),
            PartitionType::BiosBoot,
        ),
    ];
    for (k, v) in known {
        if &k == guid {
            return v;
        }
    }
    PartitionType::Unknown
}

pub fn parse_partition_entries(
    data: &[u8],
    count: u32,
    entry_size: u32,
    expected_crc: u32,
) -> GptResult<Vec<GptPartitionEntry>> {
    let computed = gpt_crc32(data);
    if computed != expected_crc {
        return Err(GptError::InvalidEntriesCrc);
    }

    let mut entries = Vec::new();
    let entry_size = entry_size as usize;
    for i in 0..count as usize {
        let start = i * entry_size;
        let end = start + entry_size;
        if end > data.len() {
            break;
        }
        if let Some(entry) = GptPartitionEntry::parse(&data[start..end]) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

pub fn parse_gpt<D: BlockDevice>(device: &D) -> GptResult<PartitionTable> {
    let sector_count = device.sector_count();
    if sector_count < 34 {
        return Err(GptError::NoGptHeader);
    }

    let primary_header_sector = device.read_sector(1)?;
    let primary_header = GptHeader::parse(primary_header_sector.as_bytes())?;

    let entry_bytes =
        primary_header.partition_entry_count as u64 * primary_header.partition_entry_size as u64;
    let entry_sectors = entry_bytes.div_ceil(device.sector_size().bytes() as u64);
    let entries_buf = device.read_sectors(primary_header.partition_entry_lba, entry_sectors)?;
    let entries = parse_partition_entries(
        entries_buf.as_bytes(),
        primary_header.partition_entry_count,
        primary_header.partition_entry_size,
        primary_header.partition_array_crc32,
    )?;

    let mut warnings = Vec::new();
    if sector_count > 2 {
        let backup_header_sector = device.read_sector(sector_count - 1)?;
        match GptHeader::parse(backup_header_sector.as_bytes()) {
            Ok(backup) => {
                if backup.disk_guid != primary_header.disk_guid {
                    warnings.push("primary/backup disk GUID mismatch".to_string());
                }
                if backup.partition_array_crc32 != primary_header.partition_array_crc32 {
                    return Err(GptError::PrimaryBackupMismatch(
                        "partition array CRC mismatch".to_string(),
                    ));
                }
            }
            Err(e) => warnings.push(format!("backup GPT header invalid: {e}")),
        }
    }

    let sector_size = device.sector_size();
    let partitions: Vec<PartitionEntry> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| e.to_partition_entry(i as u32, sector_size))
        .collect();

    Ok(PartitionTable {
        kind: PartitionTableKind::Gpt,
        boot_mode: BootMode::Uefi,
        partitions,
        disk_guid: Some(primary_header.disk_guid),
        warnings,
    })
}
