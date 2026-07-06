use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePathKind {
    ImageFile,
    LinuxBlockDevice,
    WindowsPhysicalDrive,
    Unknown,
}

pub fn classify_path(path: &str) -> DevicePathKind {
    let p = Path::new(path);
    let s = path.replace('\\', "/");

    if p.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("img"))
        .unwrap_or(false)
    {
        return DevicePathKind::ImageFile;
    }

    if s.starts_with("/dev/") {
        return DevicePathKind::LinuxBlockDevice;
    }

    if s.contains("PhysicalDrive") || s.starts_with("\\\\.\\") {
        return DevicePathKind::WindowsPhysicalDrive;
    }

    if p.is_file() {
        return DevicePathKind::ImageFile;
    }

    DevicePathKind::Unknown
}

pub fn linux_block_name(path: &str) -> Option<String> {
    let name = Path::new(path).file_name()?.to_str()?;
    if is_allowed_linux_block_name(name) {
        Some(name.to_string())
    } else {
        None
    }
}

pub fn is_allowed_linux_block_name(name: &str) -> bool {
    name.starts_with("sd")
        || name.starts_with("nvme")
        || name.starts_with("mmcblk")
        || name.starts_with("vd")
}

pub fn is_linux_disk_write_candidate(path: &str) -> bool {
    let Some(name) = linux_block_name(path) else {
        return false;
    };
    // Whole-disk nodes only (no partition suffix like sda1 / nvme0n1p1).
    if name.starts_with("nvme") {
        return !name.contains('p');
    }
    if name.starts_with("mmcblk") {
        return !name.contains('p');
    }
    if name.starts_with("sd") || name.starts_with("vd") {
        return name.chars().skip(2).all(|c| c.is_ascii_alphabetic());
    }
    false
}
