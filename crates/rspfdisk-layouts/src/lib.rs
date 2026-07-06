//! Quick layout template engine.
//!
//! Loads TOML partition templates, generates `LayoutDraft` from
//! disk information, parses size expressions (`fill`, `fill-minus:N`,
//! `auto:swap`), and produces diff reports against existing tables.
//!
//! Templates: Windows UEFI/Legacy, macOS APFS, Linux ext4/btrfs.

pub mod diff;
pub mod engine;
pub mod error;
pub mod size;
pub mod template;

pub use size::parse_byte_size;

pub use diff::build_diff_report;
pub use engine::generate_layout;
pub use error::{LayoutError, LayoutResult};
pub use template::{load_template, LayoutTemplate, TemplateRegistry};
