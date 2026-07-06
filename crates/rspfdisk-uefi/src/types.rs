use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GptPartition {
    pub index: usize,
    pub name: alloc::string::String,
    pub start_lba: u64,
    pub end_lba: u64,
    pub type_guid: [u8; 16],
}

impl fmt::Display for GptPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}  LBA {}-{}  type={:02x}{:02x}...",
            self.index,
            self.name,
            self.start_lba,
            self.end_lba,
            self.type_guid[0],
            self.type_guid[1]
        )
    }
}

#[derive(Debug, Clone)]
pub struct GptTable {
    pub disk_guid: [u8; 16],
    pub partitions: alloc::vec::Vec<GptPartition>,
}
