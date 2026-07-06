use rspfdisk_core::DiskInfo;
use rspfdisk_disk::{classify_path, is_linux_disk_write_candidate, DevicePathKind};

use crate::error::{SafetyError, SafetyResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub is_system_disk: bool,
    pub is_removable: bool,
    pub is_image: bool,
    pub warnings: Vec<String>,
    pub write_allowed: bool,
}

pub fn assess_disk(path: &str, info: &DiskInfo) -> RiskAssessment {
    let kind = classify_path(path);
    let is_image = kind == DevicePathKind::ImageFile;
    let mut warnings = Vec::new();
    let mut level = if is_image {
        RiskLevel::Low
    } else {
        RiskLevel::High
    };

    if !is_image && !info.removable {
        warnings.push("固定式磁碟：寫入可能影響系統資料".to_string());
        level = RiskLevel::High;
    }

    if info.removable {
        warnings.push("可卸除媒體".to_string());
        if !is_image {
            level = RiskLevel::Medium;
        }
    }

    if info.read_only {
        warnings.push("裝置目前為唯讀".to_string());
        level = RiskLevel::Critical;
    }

    let is_system = detect_system_disk(path, info);
    if is_system {
        warnings.push("偵測到可能為系統碟或 root 所在磁碟".to_string());
        level = RiskLevel::Critical;
    }

    if kind == DevicePathKind::LinuxBlockDevice && !is_linux_disk_write_candidate(path) {
        warnings.push("路徑不是整顆磁碟節點，寫入已封鎖".to_string());
        level = RiskLevel::Critical;
    }

    let write_allowed = is_image
        || (kind == DevicePathKind::LinuxBlockDevice
            && is_linux_disk_write_candidate(path)
            && !info.read_only);

    RiskAssessment {
        level,
        is_system_disk: is_system,
        is_removable: info.removable,
        is_image,
        warnings,
        write_allowed,
    }
}

pub fn validate_write_risk(
    path: &str,
    info: &DiskInfo,
    accept_system_disk_risk: bool,
) -> SafetyResult<RiskAssessment> {
    let assessment = assess_disk(path, info);

    if !assessment.write_allowed {
        return Err(SafetyError::WriteNotAllowed(assessment.warnings.join("; ")));
    }

    if assessment.is_system_disk && !accept_system_disk_risk {
        return Err(SafetyError::SystemDiskBlocked);
    }

    Ok(assessment)
}

fn detect_system_disk(path: &str, info: &DiskInfo) -> bool {
    if classify_path(path) == DevicePathKind::ImageFile {
        return false;
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(root_dev) = read_root_block_device() {
            let target = block_device_key(path);
            let root_key = block_device_key(&root_dev);
            if !target.is_empty() && target == root_key {
                return true;
            }
        }
    }

    let _ = info;
    false
}

#[cfg(target_os = "linux")]
fn read_root_block_device() -> Result<String, std::io::Error> {
    let mounts = std::fs::read_to_string("/proc/mounts")?;
    for line in mounts.lines() {
        let mut parts = line.split_whitespace();
        let mount_point = parts.nth(1).unwrap_or("");
        if mount_point == "/" {
            if let Some(dev) = line.split_whitespace().next() {
                return Ok(dev.to_string());
            }
        }
    }
    Ok(String::new())
}

#[cfg(target_os = "linux")]
fn block_device_key(path: &str) -> String {
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    if let Some(base) = name.strip_prefix("nvme") {
        if let Some(idx) = base.find('p') {
            return format!("nvme{}", &base[..idx]);
        }
    }
    if let Some(idx) = name.find('p') {
        if name.starts_with("mmcblk") {
            return name[..idx].to_string();
        }
        if name.starts_with("sd") || name.starts_with("vd") {
            return name[..idx].to_string();
        }
    }
    name.to_string()
}
