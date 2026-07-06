use rspfdisk_core::PartitionEntry;

use crate::error::{MbrError, MbrResult};

pub fn validate_partitions(partitions: &[PartitionEntry]) -> MbrResult<()> {
    for i in 0..partitions.len() {
        for j in (i + 1)..partitions.len() {
            let a = &partitions[i];
            let b = &partitions[j];
            if ranges_overlap(a.start_lba, a.end_lba, b.start_lba, b.end_lba) {
                return Err(MbrError::PartitionOverlap);
            }
        }
    }
    Ok(())
}

fn ranges_overlap(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    a_start <= b_end && b_start <= a_end
}
