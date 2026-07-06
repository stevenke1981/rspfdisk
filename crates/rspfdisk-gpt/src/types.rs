pub const GPT_SIGNATURE: &[u8; 8] = b"EFI PART";
pub const GPT_HEADER_SIZE: usize = 92;
pub const GPT_ENTRY_SIZE: usize = 128;
pub const GPT_ENTRY_COUNT: u32 = 128;
pub const GPT_PARTITION_NAME_BYTES: usize = 72;
pub const GPT_FIRST_USABLE_LBA: u64 = 34;
