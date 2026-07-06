//! no_std GPT parser for UEFI PoC.
//!
//! Designed for `x86_64-unknown-uefi` target. Reads GPT from
//! UEFI Block IO protocol. Also unit-testable on host (`cargo test`).
//!
//! Limitations: read-only, no TUI, no write support yet.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod gpt;
pub mod types;

pub use gpt::parse_gpt_from_disk_sectors;
pub use types::{GptPartition, GptTable};
