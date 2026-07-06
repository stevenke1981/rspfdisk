pub const MBR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
pub const MBR_BOOTCODE_SIZE: usize = 446;
pub const MBR_ENTRY_COUNT: usize = 4;
pub const MBR_ENTRY_SIZE: usize = 16;
pub const PARTITION_TYPE_GPT_PROTECTIVE: u8 = 0xEE;
pub const PARTITION_TYPE_EXTENDED: u8 = 0x05;
pub const PARTITION_TYPE_EXTENDED_LBA: u8 = 0x0F;
