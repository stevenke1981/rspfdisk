//! GPT partition table parser, validator, and writer.
//!
//! Parses GPT header (primary + backup), validates CRC,
//! decodes UTF-16LE partition names, detects inconsistencies.
//! `write_gpt_from_draft()` writes a complete GPT from a `LayoutDraft`.
//! `validate_alignment()` checks 1 MiB alignment.
//!
//! Also exports [`partition_type_guid()`] for GPT type GUID mapping.

pub mod crc;
pub mod error;
pub mod guid;
pub mod header;
pub mod parser;
pub mod types;
pub mod validator;
pub mod writer;

pub use error::{GptError, GptResult};
pub use guid::partition_type_guid;
pub use header::GptHeader;
pub use parser::parse_gpt;
pub use validator::{validate_alignment, validate_gpt};
pub use writer::write_gpt_from_draft;
