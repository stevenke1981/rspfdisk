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

    let mut exprs = Vec::with_capacity(template.partitions.len());
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

        exprs.push(parse_size_expr(&size_str)?);
    }

    let mut current_lba = first_usable;
    let mut partitions = Vec::new();

    for (idx, part) in template.partitions.iter().enumerate() {
        current_lba = sector_size.align_lba(current_lba, ALIGN_1MIB);
        if current_lba > last_usable {
            return Err(CoreError::InsufficientSpace(format!(
                "no usable space left for partition '{}'",
                part.name
            )));
        }

        let available_bytes = (last_usable - current_lba + 1) * sector_size.bytes() as u64;
        let size = match exprs[idx].clone() {
            SizeExpr::Fixed(bytes) => align_size(bytes, sector_size),
            SizeExpr::AutoSwap => align_size(auto_swap_size_bytes(), sector_size),
            SizeExpr::Fill => largest_size_that_fits_tail(
                current_lba,
                available_bytes,
                &exprs[idx + 1..],
                sector_size,
                last_usable,
            )?,
            SizeExpr::FillMinus(minus) => {
                let minus = align_size(minus, sector_size);
                if available_bytes <= minus {
                    return Err(CoreError::InsufficientSpace(format!(
                        "fill-minus needs {minus} bytes reserved but only {available_bytes} bytes remain"
                    )));
                }
                largest_size_that_fits_tail(
                    current_lba,
                    available_bytes - minus,
                    &exprs[idx + 1..],
                    sector_size,
                    last_usable,
                )?
            }
        };

        if size == 0 {
            return Err(CoreError::InsufficientSpace(format!(
                "partition '{}' computed zero size",
                part.name
            )));
        }
        if size > available_bytes {
            return Err(CoreError::InsufficientSpace(format!(
                "partition '{}' needs {size} bytes but only {available_bytes} bytes remain",
                part.name
            )));
        }

        let size_sectors = bytes_to_sectors(size, sector_size);

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

fn largest_size_that_fits_tail(
    start_lba: u64,
    upper_bytes: u64,
    tail: &[SizeExpr],
    sector_size: SectorSize,
    last_usable: u64,
) -> CoreResult<u64> {
    let sector_bytes = sector_size.bytes() as u64;
    let mut low = 0u64;
    let mut high = upper_bytes / sector_bytes;

    while low < high {
        let mid = (low + high).div_ceil(2);
        if mid > 0 && tail_fits_after(start_lba + mid, tail, sector_size, last_usable)? {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    if low == 0 {
        return Err(CoreError::InsufficientSpace(
            "no space left for fill partition".to_string(),
        ));
    }

    Ok(low * sector_bytes)
}

fn tail_fits_after(
    mut current_lba: u64,
    tail: &[SizeExpr],
    sector_size: SectorSize,
    last_usable: u64,
) -> CoreResult<bool> {
    for expr in tail {
        current_lba = sector_size.align_lba(current_lba, ALIGN_1MIB);
        if current_lba > last_usable {
            return Ok(false);
        }

        let min_size = match expr {
            SizeExpr::Fixed(bytes) => align_size(*bytes, sector_size),
            SizeExpr::AutoSwap => align_size(auto_swap_size_bytes(), sector_size),
            SizeExpr::Fill | SizeExpr::FillMinus(_) => sector_size.bytes() as u64,
        };
        let size_sectors = bytes_to_sectors(min_size, sector_size);
        let end_lba = current_lba
            .checked_add(size_sectors - 1)
            .ok_or_else(|| CoreError::InsufficientSpace("LBA range overflow".to_string()))?;
        if end_lba > last_usable {
            return Ok(false);
        }
        current_lba = end_lba + 1;
    }

    Ok(true)
}
