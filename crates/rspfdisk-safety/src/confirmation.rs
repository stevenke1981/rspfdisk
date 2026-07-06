use std::path::Path;

use rspfdisk_core::ChangePlan;
use rspfdisk_disk::classify_path;
use rspfdisk_disk::DevicePathKind;
use serde_json;

use crate::danger::validate_write_risk;
use crate::error::{SafetyError, SafetyResult};
use crate::token::WriteToken;

#[derive(Debug, Clone)]
pub struct ConfirmationOptions {
    pub write: bool,
    pub dry_run: bool,
    pub image_confirmed: bool,
    pub confirmation_phrase: Option<String>,
    pub backup_path: Option<String>,
    pub accept_system_disk_risk: bool,
}

pub fn confirm_write(
    plan: &ChangePlan,
    opts: &ConfirmationOptions,
    disk_info: &rspfdisk_core::DiskInfo,
) -> SafetyResult<WriteToken> {
    if opts.dry_run {
        return Err(SafetyError::WriteFlagRequired);
    }
    if !opts.write {
        return Err(SafetyError::WriteFlagRequired);
    }

    let path = Path::new(&plan.disk_path);
    let is_image = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("img"))
        .unwrap_or(false);

    if is_image && !opts.image_confirmed {
        return Err(SafetyError::ImageConfirmationRequired);
    }

    if classify_path(&plan.disk_path) == DevicePathKind::LinuxBlockDevice {
        validate_write_risk(&plan.disk_path, disk_info, opts.accept_system_disk_risk)?;
    }

    if opts.backup_path.is_none() {
        return Err(SafetyError::BackupRequired);
    }

    let expected_phrase = disk_confirmation_phrase(&plan.disk_path);
    match &opts.confirmation_phrase {
        Some(phrase) if phrase == &expected_phrase => {}
        Some(_) => return Err(SafetyError::PhraseMismatch),
        None if is_image => {}
        None => return Err(SafetyError::RealDiskConfirmRequired),
    }

    let plan_json = serde_json::to_string(plan).unwrap_or_default();
    Ok(WriteToken::new(
        plan.disk_path.clone(),
        opts.backup_path.clone(),
        opts.confirmation_phrase.as_deref().unwrap_or("image"),
        &plan_json,
    ))
}

pub fn disk_confirmation_phrase(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}
