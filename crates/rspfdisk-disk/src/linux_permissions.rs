use crate::error::{DiskError, DiskResult};

pub fn is_root() -> bool {
    // SAFETY: geteuid is always safe to call.
    unsafe { libc::geteuid() == 0 }
}

pub fn require_root_for_write() -> DiskResult<()> {
    if is_root() {
        Ok(())
    } else {
        Err(DiskError::InsufficientPrivileges(
            "writing block devices requires root (run with sudo)".to_string(),
        ))
    }
}
