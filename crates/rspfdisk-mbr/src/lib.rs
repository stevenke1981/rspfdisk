//! MBR / EBR partition table parser and writer.
//!
//! Parses MBR sector, 4 primary entries, EBR logical chain.
//! Validates signatures, detects protective MBR.
//! Write support is limited to protective MBR for GPT images.

pub mod entry;
pub mod error;
pub mod parser;
pub mod types;
pub mod validator;
pub mod writer;

pub use error::{MbrError, MbrResult};
pub use parser::{parse_mbr, parse_mbr_sector};
pub use validator::validate_partitions;
pub use writer::write_protective_mbr;
