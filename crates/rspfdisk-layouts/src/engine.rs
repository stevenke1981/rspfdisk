use rspfdisk_core::{
    BootMode, CoreError, CoreResult, DiskInfo, LayoutDraft, PartitionDraft, PartitionTableKind,
    PartitionType, SectorSize, ALIGN_1MIB,
};

use crate::size::{
    align_size, auto_swap_size_bytes, bytes_to_sectors, parse_byte_size, parse_size_expr, SizeExpr,
};
use crate::template::LayoutTemplate;

const GPT_RESERVED_SECTORS: u64 = 34;

pub fn generate_layout(
    template: &LayoutTemplate,
    disk: &DiskInfo,
    root_size_override: Option<u64>,
) -> CoreResult<LayoutDraft> {
    let sector_size = disk.logical_sector_size;
    let min_disk = parse_byte_size(&template.min_disk_size)?;
    if disk.size_bytes < min_disk {
        return Err(CoreError::DiskTooSmall {
            need: min_disk,
            have: disk.size_bytes,
        });
    }

    let first_usable = gpt_first_usable_lba(sector_size);
    let last_usable = disk.size_bytes / sector_size.bytes() as u64 - GPT_RESERVED_SECTORS;
    let usable_bytes = (last_usable - first_usable + 1) * sector_size.bytes() as u64;

    let boot_mode = match template.boot_mode.as_str() {
        "uefi" => BootMode::Uefi,
        "bios" => BootMode::Bios,
        _ => BootMode::Unknown,
    };

    let table = match template.table.as_str() {
        "gpt" => PartitionTableKind::Gpt,
        "mbr" => PartitionTableKind::Mbr,
        _ => PartitionTableKind::Unknown,
    };

    let mut sizes: Vec<u64> = Vec::with_capacity(template.partitions.len());
    let mut remaining = usable_bytes;

    for part in &template.partitions {
        let size_str = if part.name == "Linux Root" {
            if let Some(override_size) = root_size_override {
                override_size.to_string()
            } else {
                part.size.clone()
            }
        } else {
            part.size.clone()
        };

        let expr = parse_size_expr(&size_str)?;
        let size = match expr {
            SizeExpr::Fixed(bytes) => align_size(bytes, sector_size),
            SizeExpr::AutoSwap => align_size(auto_swap_size_bytes(), sector_size),
            SizeExpr::Fill => {
                if remaining == 0 {
                    return Err(CoreError::InsufficientSpace(
                        "no space left for fill partition".to_string(),
                    ));
                }
                remaining
            }
            SizeExpr::FillMinus(minus) => {
                let minus = align_size(minus, sector_size);
                if remaining <= minus {
                    return Err(CoreError::InsufficientSpace(format!(
                        "fill-minus needs {minus} bytes reserved but only {remaining} bytes remain"
                    )));
                }
                remaining - minus
            }
        };

        if size == 0 {
            return Err(CoreError::InsufficientSpace(format!(
                "partition '{}' computed zero size",
                part.name
            )));
        }
        if size > remaining {
            return Err(CoreError::InsufficientSpace(format!(
                "partition '{}' needs {size} bytes but only {remaining} bytes remain",
                part.name
            )));
        }

        sizes.push(size);
        remaining -= size;
    }

    let mut current_lba = first_usable;
    let mut partitions = Vec::new();

    for (part, size_bytes) in template.partitions.iter().zip(sizes.iter()) {
        let size_sectors = bytes_to_sectors(*size_bytes, sector_size);
        current_lba = sector_size.align_lba(current_lba, ALIGN_1MIB);

        partitions.push(PartitionDraft {
            name: part.name.clone(),
            start_lba: current_lba,
            size_bytes: size_sectors * sector_size.bytes() as u64,
            partition_type: PartitionType::from_slug(&part.partition_type),
            filesystem: part.filesystem.clone(),
            mount_point: part.mount.clone(),
            note: part.note.clone(),
            flags: part.flags.clone().unwrap_or_default(),
        });

        current_lba += size_sectors;
    }

    Ok(LayoutDraft {
        template_name: template.name.clone(),
        display_name: template.display_name.clone(),
        table,
        boot_mode,
        partitions,
    })
}

fn gpt_first_usable_lba(sector_size: SectorSize) -> u64 {
    sector_size.align_lba(GPT_RESERVED_SECTORS, ALIGN_1MIB)
}
