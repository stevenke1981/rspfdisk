//! Backup and restore in `.rspbak` format.
//!
//! Creates disk backups with SHA256 verification, partition manifest,
//! and metadata. Supports dry-run restore with identity checking.

pub mod error;
pub mod format;
pub mod restore;
pub mod writer;

pub use error::{BackupError, BackupResult};
pub use format::BackupManifest;
pub use restore::{restore_dry_run, RestoreDiff};
pub use writer::create_backup;
