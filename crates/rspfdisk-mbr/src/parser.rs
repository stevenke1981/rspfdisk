use rspfdisk_core::{BootMode, PartitionTable, PartitionTableKind};
use rspfdisk_disk::BlockDevice;

use crate::entry::MbrEntry;
use crate::error::{MbrError, MbrResult};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct MbrTable {
    pub entries: [Option<MbrEntry>; MBR_ENTRY_COUNT],
    pub valid_signature: bool,
    pub is_empty: bool,
    pub is_protective: bool,
}

pub fn parse_mbr_sector(data: &[u8]) -> MbrResult<MbrTable> {
    if data.len() < 512 {
        return Err(MbrError::InvalidEntry);
    }
    let valid_signature = data[510] == MBR_SIGNATURE[0] && data[511] == MBR_SIGNATURE[1];
    let mut entries = [None; MBR_ENTRY_COUNT];
    let mut has_any = false;
    let mut is_protective = false;

    for (i, entry) in entries.iter_mut().enumerate() {
        let start = MBR_BOOTCODE_SIZE + i * MBR_ENTRY_SIZE;
        let end = start + MBR_ENTRY_SIZE;
        if let Some(parsed) = MbrEntry::parse(&data[start..end]) {
            has_any = true;
            if parsed.partition_type == PARTITION_TYPE_GPT_PROTECTIVE {
                is_protective = true;
            }
            *entry = Some(parsed);
        }
    }

    Ok(MbrTable {
        entries,
        valid_signature,
        is_empty: !has_any,
        is_protective,
    })
}

pub fn parse_mbr<D: BlockDevice>(device: &D) -> MbrResult<PartitionTable> {
    let sector = device.read_sector(0)?;
    let mbr = parse_mbr_sector(sector.as_bytes())?;
    let sector_size = device.sector_size().bytes();

    if !mbr.valid_signature && !mbr.is_empty {
        return Err(MbrError::InvalidSignature);
    }

    let mut partitions = Vec::new();
    for (i, entry) in mbr.entries.iter().enumerate() {
        if let Some(e) = entry {
            partitions.push(e.to_partition_entry(i as u32, sector_size));
        }
    }

    let boot_mode = if mbr.is_protective {
        BootMode::Uefi
    } else if mbr.is_empty {
        BootMode::Unknown
    } else {
        BootMode::Bios
    };

    Ok(PartitionTable {
        kind: if mbr.is_protective {
            PartitionTableKind::Gpt
        } else if mbr.is_empty {
            PartitionTableKind::Unknown
        } else {
            PartitionTableKind::Mbr
        },
        boot_mode,
        partitions,
        disk_guid: None,
        warnings: Vec::new(),
    })
}
