//! no_std GPT parser for UEFI PoC (also unit-tested on host).

#![cfg_attr(not(test), no_std)]

extern crate alloc;

pub mod gpt;
pub mod types;

pub use gpt::parse_gpt_from_disk_sectors;
pub use types::{GptPartition, GptTable};
