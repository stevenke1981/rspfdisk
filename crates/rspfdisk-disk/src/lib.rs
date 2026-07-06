//! Block device abstractions for rust-spfdisk.

pub mod device;
pub mod device_handle;
pub mod error;
pub mod file_device;
pub mod open;
pub mod path_kind;
pub mod readonly;
pub mod scan;
pub mod sector_buf;
pub mod test_helpers;

#[cfg(target_os = "linux")]
mod linux_device;
#[cfg(target_os = "linux")]
mod linux_permissions;
#[cfg(target_os = "linux")]
mod linux_scan;
#[cfg(target_os = "linux")]
mod linux_sysfs;

pub use device::{BlockDevice, WritableBlockDevice};
pub use device_handle::DeviceHandle;
pub use error::{DiskError, DiskResult};
pub use file_device::FileBlockDevice;
pub use open::{open_read_only, open_read_write};
pub use path_kind::{
    classify_path, is_linux_disk_write_candidate, linux_block_name, DevicePathKind,
};
pub use readonly::ReadOnlyDevice;
pub use scan::list_block_devices;
pub use sector_buf::SectorBuf;
pub use test_helpers::create_test_image;
