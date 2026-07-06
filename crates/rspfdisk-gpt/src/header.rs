use uuid::Uuid;

use crate::crc::gpt_crc32;
use crate::error::{GptError, GptResult};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct GptHeader {
    pub revision: u32,
    pub header_size: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: Uuid,
    pub partition_entry_lba: u64,
    pub partition_entry_count: u32,
    pub partition_entry_size: u32,
    pub partition_array_crc32: u32,
    pub header_crc32: u32,
}

impl GptHeader {
    pub fn parse(data: &[u8]) -> GptResult<Self> {
        if data.len() < 512 {
            return Err(GptError::NoGptHeader);
        }
        if &data[0..8] != GPT_SIGNATURE {
            return Err(GptError::InvalidSignature);
        }

        let header_size = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let stored_crc = u32::from_le_bytes(data[16..20].try_into().unwrap());

        // Validate header_size before using it as a slice bound.
        // Minimum valid GPT header is 92 bytes; maximum fits in a sector.
        if header_size < 92 || header_size > data.len() as u32 {
            return Err(GptError::NoGptHeader);
        }

        let mut crc_buf = data[..header_size as usize].to_vec();
        crc_buf[16..20].copy_from_slice(&0u32.to_le_bytes());
        let computed_crc = gpt_crc32(&crc_buf);
        if stored_crc != computed_crc {
            return Err(GptError::InvalidHeaderCrc);
        }

        // Safety: all slice ranges are within [0..92) which is ≤ the validated
        // header_size (≥ 92) and data is a full 512-byte sector.
        Ok(Self {
            revision: u32::from_le_bytes(data[8..12].try_into().unwrap()),
            header_size,
            current_lba: u64::from_le_bytes(data[32..40].try_into().unwrap()),
            backup_lba: u64::from_le_bytes(data[40..48].try_into().unwrap()),
            first_usable_lba: u64::from_le_bytes(data[48..56].try_into().unwrap()),
            last_usable_lba: u64::from_le_bytes(data[56..64].try_into().unwrap()),
            disk_guid: Uuid::from_bytes(data[64..80].try_into().unwrap()),
            partition_entry_lba: u64::from_le_bytes(data[80..88].try_into().unwrap()),
            partition_entry_count: u32::from_le_bytes(data[88..92].try_into().unwrap()),
            partition_entry_size: u32::from_le_bytes(data[92..96].try_into().unwrap()),
            partition_array_crc32: u32::from_le_bytes(data[96..100].try_into().unwrap()),
            header_crc32: stored_crc,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![0u8; 512];
        data[0..8].copy_from_slice(GPT_SIGNATURE);
        data[8..12].copy_from_slice(&self.revision.to_le_bytes());
        data[12..16].copy_from_slice(&self.header_size.to_le_bytes());
        data[32..40].copy_from_slice(&self.current_lba.to_le_bytes());
        data[40..48].copy_from_slice(&self.backup_lba.to_le_bytes());
        data[48..56].copy_from_slice(&self.first_usable_lba.to_le_bytes());
        data[56..64].copy_from_slice(&self.last_usable_lba.to_le_bytes());
        data[64..80].copy_from_slice(self.disk_guid.as_bytes());
        data[80..88].copy_from_slice(&self.partition_entry_lba.to_le_bytes());
        data[88..92].copy_from_slice(&self.partition_entry_count.to_le_bytes());
        data[92..96].copy_from_slice(&self.partition_entry_size.to_le_bytes());
        data[96..100].copy_from_slice(&self.partition_array_crc32.to_le_bytes());

        let crc = gpt_crc32(&data[..self.header_size as usize]);
        data[16..20].copy_from_slice(&crc.to_le_bytes());
        data
    }
}
