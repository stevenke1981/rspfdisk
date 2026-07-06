use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{GptPartition, GptTable};

const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
const GPT_ENTRY_SIZE: usize = 128;
const GPT_NAME_BYTES: usize = 72;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GptError {
    InvalidSignature,
    InvalidHeaderCrc,
    InvalidEntriesCrc,
    BufferTooSmall,
    NoPartitions,
}

pub fn gpt_crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

pub fn parse_gpt_from_disk_sectors(
    header_lba: &[u8],
    entries_bytes: &[u8],
) -> Result<GptTable, GptError> {
    if header_lba.len() < 512 {
        return Err(GptError::BufferTooSmall);
    }
    if &header_lba[0..8] != GPT_SIGNATURE {
        return Err(GptError::InvalidSignature);
    }

    let header_size = u32::from_le_bytes(header_lba[12..16].try_into().unwrap()) as usize;
    if header_lba.len() < header_size {
        return Err(GptError::BufferTooSmall);
    }

    let stored_crc = u32::from_le_bytes(header_lba[16..20].try_into().unwrap());
    let mut crc_buf = header_lba[..header_size].to_vec();
    crc_buf[16..20].copy_from_slice(&0u32.to_le_bytes());
    if gpt_crc32(&crc_buf) != stored_crc {
        return Err(GptError::InvalidHeaderCrc);
    }

    let entry_count = u32::from_le_bytes(header_lba[88..92].try_into().unwrap());
    let entry_size = u32::from_le_bytes(header_lba[92..96].try_into().unwrap()) as usize;
    let entries_crc = u32::from_le_bytes(header_lba[96..100].try_into().unwrap());
    if gpt_crc32(entries_bytes) != entries_crc {
        return Err(GptError::InvalidEntriesCrc);
    }

    let mut disk_guid = [0u8; 16];
    disk_guid.copy_from_slice(&header_lba[64..80]);

    let mut partitions = Vec::new();
    for i in 0..entry_count as usize {
        let start = i * entry_size;
        let end = start + entry_size;
        if end > entries_bytes.len() {
            break;
        }
        if let Some(part) = parse_entry(&entries_bytes[start..end], i + 1) {
            partitions.push(part);
        }
    }

    if partitions.is_empty() {
        return Err(GptError::NoPartitions);
    }

    Ok(GptTable {
        disk_guid,
        partitions,
    })
}

fn parse_entry(raw: &[u8], index: usize) -> Option<GptPartition> {
    if raw.len() < GPT_ENTRY_SIZE {
        return None;
    }
    let mut type_guid = [0u8; 16];
    type_guid.copy_from_slice(&raw[0..16]);
    if type_guid == [0u8; 16] {
        return None;
    }

    let start_lba = u64::from_le_bytes(raw[32..40].try_into().ok()?);
    let end_lba = u64::from_le_bytes(raw[40..48].try_into().ok()?);
    if start_lba == 0 && end_lba == 0 {
        return None;
    }

    Some(GptPartition {
        index,
        name: decode_utf16_name(&raw[56..56 + GPT_NAME_BYTES]),
        start_lba,
        end_lba,
        type_guid,
    })
}

fn decode_utf16_name(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks_exact(2) {
        let unit = u16::from_le_bytes([chunk[0], chunk[1]]);
        if unit == 0 {
            break;
        }
        if let Some(ch) = char::from_u32(unit as u32) {
            out.push(ch);
        }
    }
    if out.is_empty() {
        out.push_str("(unnamed)");
    }
    out
}
