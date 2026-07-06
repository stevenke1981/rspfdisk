use rspfdisk_core::{SectorSize, ALIGN_1MIB};

use crate::error::{GptError, GptResult};
use crate::parser::GptPartitionEntry;

pub fn validate_gpt(partitions: &[GptPartitionEntry]) -> GptResult<()> {
    for i in 0..partitions.len() {
        for j in (i + 1)..partitions.len() {
            let a = &partitions[i];
            let b = &partitions[j];
            if a.first_lba <= b.last_lba && b.first_lba <= a.last_lba {
                return Err(GptError::PartitionOverlap);
            }
        }
    }
    Ok(())
}

pub fn validate_alignment(lba: u64, sector_size: SectorSize) -> GptResult<()> {
    let aligned = sector_size.align_lba(lba, ALIGN_1MIB);
    if aligned != lba {
        return Err(GptError::AlignmentViolation(format!(
            "LBA {lba} not 1MiB aligned"
        )));
    }
    Ok(())
}
