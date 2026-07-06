use rspfdisk_core::{LayoutDraft, ALIGN_1MIB};
use rspfdisk_disk::{SectorBuf, WritableBlockDevice};
use rspfdisk_mbr::write_protective_mbr;
use uuid::Uuid;

use crate::crc::gpt_crc32;
use crate::guid::{encode_utf16le_name, partition_type_guid, write_guid_le};
use crate::header::GptHeader;
use crate::types::*;
use crate::GptResult;

pub fn write_gpt_from_draft<D: WritableBlockDevice>(
    device: &mut D,
    draft: &LayoutDraft,
) -> GptResult<()> {
    write_protective_mbr(device)?;

    let sector_size = device.sector_size();
    let sector_count = device.sector_count();
    let disk_guid = Uuid::new_v4();
    let last_usable = sector_count - 34;
    let first_usable = sector_size.align_lba(34, ALIGN_1MIB);

    let mut entries_data = vec![0u8; GPT_ENTRY_COUNT as usize * GPT_ENTRY_SIZE];
    let mut current_lba = first_usable;

    for (i, part) in draft.partitions.iter().enumerate() {
        let size_sectors = part.size_bytes / sector_size.bytes() as u64;
        let start = i * GPT_ENTRY_SIZE;
        let entry = &mut entries_data[start..start + GPT_ENTRY_SIZE];

        let type_guid = partition_type_guid(&part.partition_type);
        write_guid_le(&type_guid, &mut entry[0..16]);
        write_guid_le(&Uuid::new_v4(), &mut entry[16..32]);
        entry[32..40].copy_from_slice(&current_lba.to_le_bytes());
        let end_lba = current_lba + size_sectors - 1;
        entry[40..48].copy_from_slice(&end_lba.to_le_bytes());

        let attrs = if part.flags.iter().any(|f| f == "esp" || f == "boot") {
            0x1u64
        } else {
            0
        };
        entry[48..56].copy_from_slice(&attrs.to_le_bytes());
        encode_utf16le_name(&part.name, &mut entry[56..56 + GPT_PARTITION_NAME_BYTES]);

        current_lba = sector_size.align_lba(end_lba + 1, ALIGN_1MIB);
    }

    let entries_crc = gpt_crc32(&entries_data);
    let entry_lba = 2u64;
    let entry_sector_count = 32u64;

    let primary = GptHeader {
        revision: 0x0001_0000,
        header_size: GPT_HEADER_SIZE as u32,
        current_lba: 1,
        backup_lba: sector_count - 1,
        first_usable_lba: first_usable,
        last_usable_lba: last_usable,
        disk_guid,
        partition_entry_lba: entry_lba,
        partition_entry_count: GPT_ENTRY_COUNT,
        partition_entry_size: GPT_ENTRY_SIZE as u32,
        partition_array_crc32: entries_crc,
        header_crc32: 0,
    };

    let backup = GptHeader {
        current_lba: sector_count - 1,
        backup_lba: 1,
        partition_entry_lba: sector_count - 33,
        ..primary.clone()
    };

    write_header(device, 1, &primary)?;
    write_entries(device, entry_lba, entry_sector_count, &entries_data)?;
    write_header(device, sector_count - 1, &backup)?;
    write_entries(device, sector_count - 33, entry_sector_count, &entries_data)?;
    device.flush()?;
    Ok(())
}

fn write_header<D: WritableBlockDevice>(
    device: &mut D,
    lba: u64,
    header: &GptHeader,
) -> GptResult<()> {
    let data = header.serialize();
    let mut sector = SectorBuf::new(device.sector_size(), 1);
    sector.as_bytes_mut()[..data.len()].copy_from_slice(&data);
    device.write_sector(lba, &sector)?;
    Ok(())
}

fn write_entries<D: WritableBlockDevice>(
    device: &mut D,
    lba: u64,
    sector_count: u64,
    entries: &[u8],
) -> GptResult<()> {
    let mut sector = SectorBuf::new(device.sector_size(), sector_count as usize);
    let len = entries.len().min(sector.as_bytes().len());
    sector.as_bytes_mut()[..len].copy_from_slice(&entries[..len]);
    device.write_sectors(lba, &sector)?;
    Ok(())
}
