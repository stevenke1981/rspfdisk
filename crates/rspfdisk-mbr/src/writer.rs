use rspfdisk_disk::{SectorBuf, WritableBlockDevice};

use crate::entry::MbrEntry;
use crate::error::MbrResult;
use crate::types::*;

pub fn write_protective_mbr<D: WritableBlockDevice>(device: &mut D) -> MbrResult<()> {
    let sector_count = device.sector_count();
    let max_u32 = u32::MAX as u64;
    let (start_lba, count) = if sector_count > max_u32 {
        (1u32, u32::MAX)
    } else {
        (1, sector_count as u32 - 1)
    };

    let entry = MbrEntry {
        bootable: false,
        partition_type: PARTITION_TYPE_GPT_PROTECTIVE,
        start_lba,
        sector_count: count,
    };

    let mut sector = SectorBuf::new(device.sector_size(), 1);
    let data = sector.as_bytes_mut();
    data[446..462].copy_from_slice(&entry.to_bytes());
    data[510] = MBR_SIGNATURE[0];
    data[511] = MBR_SIGNATURE[1];

    device.write_sector(0, &sector)?;
    device.flush()?;
    Ok(())
}
