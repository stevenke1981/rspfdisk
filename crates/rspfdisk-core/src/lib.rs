//! Core data structures for rspfdisk.
//!
//! Shared types used across all crates:
//! - [`SectorSize`], [`DiskInfo`] — physical disk representation
//! - [`PartitionTable`], [`PartitionEntry`] — parsed partition tables
//! - [`LayoutDraft`], [`PartitionDraft`] — template-generated layouts
//! - [`PartitionType`] — abstract partition type slugs
//! - [`ChangePlan`] — transactional write plan
//! - [`CoreError`], [`CoreResult`] — unified error types
//! - [`BootMode`], [`PartitionTableKind`] — enumeration types
//! - [`ALIGN_1MIB`] — standard 1 MiB alignment constant

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
