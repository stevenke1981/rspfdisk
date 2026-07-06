//! MBR / EBR partition table parser and writer.

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
