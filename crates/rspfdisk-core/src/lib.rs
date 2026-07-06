//! Core data structures for rust-spfdisk.

pub mod error;
pub mod layout;
pub mod partition;
pub mod sector;

pub use error::{CoreError, CoreResult};
pub use layout::{ChangePlan, DiffReport, LayoutDraft, PartitionDraft};
pub use partition::{
    BootMode, DiskInfo, PartitionEntry, PartitionTable, PartitionTableKind, PartitionType,
};
pub use sector::{SectorSize, ALIGN_1MIB};
