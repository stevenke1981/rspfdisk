use std::path::Path;

use anyhow::{anyhow, Context, Result};
use rspfdisk_backup::{create_backup, restore_dry_run};
use rspfdisk_core::{ChangePlan, SectorSize};
use rspfdisk_disk::{
    classify_path, create_test_image, list_block_devices, open_read_only, open_read_write,
    BlockDevice, DevicePathKind,
};
use rspfdisk_gpt::{parse_gpt, validate_alignment, write_gpt_from_draft};
use rspfdisk_layouts::{build_diff_report, generate_layout, load_template, TemplateRegistry};
use rspfdisk_mbr::parse_mbr;
use rspfdisk_safety::{assess_disk, confirm_write, disk_confirmation_phrase, ConfirmationOptions};

pub fn list_disks() -> Result<()> {
    let disks = list_block_devices().map_err(|e| anyhow!("{e}"))?;

    if disks.is_empty() {
        println!("磁碟列表：目前平台未偵測到區塊裝置。");
        println!("  Linux: 列出 /dev/sd* /dev/nvme* /dev/mmcblk*");
        println!("  其他平台: 使用 image file，例如 rspfdisk inspect disk.img");
        return Ok(());
    }

    println!("磁碟列表：");
    for (i, disk) in disks.iter().enumerate() {
        let size_gib = disk.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let model = disk.model.as_deref().unwrap_or("-");
        let serial = disk.serial.as_deref().unwrap_or("-");
        let removable = if disk.removable {
            "可卸除"
        } else {
            "固定"
        };
        println!(
            "  [{}] {}  {:.2} GiB  {}  {}  model={} serial={}",
            i + 1,
            disk.path,
            size_gib,
            removable,
            disk.logical_sector_size.bytes(),
            model,
            serial
        );
    }
    Ok(())
}

pub fn inspect(path: &str, json: bool) -> Result<()> {
    let device = open_read_only(path).map_err(|e| anyhow!("{e}"))?;
    let info = device.info();
    let risk = assess_disk(path, &info);

    let gpt = parse_gpt(&device);
    let table = match gpt {
        Ok(t) => t,
        Err(_) => parse_mbr(&device).context("無法解析 MBR 或 GPT")?,
    };

    if json {
        let output = serde_json::json!({
            "disk": info,
            "risk": {
                "level": format!("{:?}", risk.level),
                "is_system_disk": risk.is_system_disk,
                "warnings": risk.warnings,
            },
            "partition_table": table,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("磁碟: {}", path);
        println!(
            "容量: {:.2} GiB  sector={}  {}",
            info.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            info.logical_sector_size.bytes(),
            if info.removable {
                "可卸除"
            } else {
                "固定"
            }
        );
        if let Some(model) = &info.model {
            println!("型號: {model}");
        }
        if let Some(serial) = &info.serial {
            println!("序號: {serial}");
        }
        println!("風險等級: {:?}", risk.level);
        for w in &risk.warnings {
            println!("  警告: {w}");
        }
        println!("分割表: {:?}", table.kind);
        println!("開機模式: {:?}", table.boot_mode);
        for part in &table.partitions {
            let size_gib = part.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            println!(
                "  [{}] {}  {:.2} GiB  {:?}  LBA {}-{}",
                part.index + 1,
                part.name,
                size_gib,
                part.partition_type,
                part.start_lba,
                part.end_lba
            );
        }
        for w in &table.warnings {
            println!("  分割表警告: {w}");
        }
    }
    Ok(())
}

pub fn backup(path: &str, out: &str) -> Result<()> {
    let device = open_read_only(path).map_err(|e| anyhow!("{e}"))?;
    let manifest = create_backup(&device, out)?;
    println!("備份完成: {out}");
    println!("分割表類型: {}", manifest.partition_table);
    Ok(())
}

pub fn restore(path: &str, backup_path: &str, dry_run: bool) -> Result<()> {
    if !dry_run {
        return Err(anyhow!("還原寫入尚未實作，請使用 --dry-run"));
    }
    let device = open_read_only(path).map_err(|e| anyhow!("{e}"))?;
    let info = device.info();
    let diff = restore_dry_run(backup_path, &info)?;
    println!("還原 dry-run:");
    println!("  identity_match: {}", diff.identity_match);
    for d in &diff.differences {
        println!("  差異: {d}");
    }
    Ok(())
}

pub struct LayoutOptions<'a> {
    pub template_name: &'a str,
    pub path: &'a str,
    pub dry_run: bool,
    pub write: bool,
    pub image_confirmed: bool,
    pub confirm_phrase: Option<&'a str>,
    pub accept_system_disk_risk: bool,
    pub root_size: Option<&'a str>,
}

pub fn layout(opts: &LayoutOptions<'_>) -> Result<()> {
    let template_name = opts.template_name;
    let path = opts.path;
    let dry_run = opts.dry_run;
    let write = opts.write;
    let image_confirmed = opts.image_confirmed;
    let confirm_phrase = opts.confirm_phrase;
    let accept_system_disk_risk = opts.accept_system_disk_risk;
    let root_size = opts.root_size;
    let template = resolve_template(template_name)?;
    let root_override = root_size
        .map(parse_root_size)
        .transpose()
        .context("解析 --root-size")?;

    ensure_target_exists(path)?;

    let device = open_read_only(path).map_err(|e| anyhow!("{e}"))?;
    let disk_info = device.info();
    let risk = assess_disk(path, &disk_info);

    if write && !risk.write_allowed {
        return Err(anyhow!("寫入已封鎖: {}", risk.warnings.join("; ")));
    }

    if !dry_run || write {
        for w in &risk.warnings {
            eprintln!("警告: {w}");
        }
        if risk.is_system_disk && !accept_system_disk_risk && write {
            return Err(anyhow!(
                "偵測到系統碟風險，寫入需加上 --accept-system-disk-risk"
            ));
        }
    }

    let current = parse_gpt(&device).unwrap_or_else(|_| rspfdisk_core::PartitionTable::empty());
    let draft = generate_layout(&template, &disk_info, root_override)?;
    let diff = build_diff_report(&current, &draft);

    for line in &diff.summary_lines {
        println!("{line}");
    }

    if dry_run || !write {
        if classify_path(path) == DevicePathKind::LinuxBlockDevice {
            println!(
                "確認文字（寫入時需加 --confirm {}）",
                disk_confirmation_phrase(path)
            );
        }
        return Ok(());
    }

    let plan = ChangePlan {
        disk_path: path.to_string(),
        layout: draft.clone(),
        diff: diff.clone(),
        backup_path: None,
        dry_run: false,
    };

    let backup_out = format!("{}.rspbak", path);
    create_backup(&device, &backup_out)?;
    println!("已建立備份: {backup_out}");

    let token = confirm_write(
        &plan,
        &ConfirmationOptions {
            write: true,
            dry_run: false,
            image_confirmed,
            confirmation_phrase: confirm_phrase.map(|s| s.to_string()),
            backup_path: Some(backup_out),
            accept_system_disk_risk,
        },
        &disk_info,
    )
    .map_err(|e| anyhow!("{e}"))?;

    let mut writable = open_read_write(path).map_err(|e| anyhow!("{e}"))?;
    write_gpt_from_draft(&mut writable, &draft)?;
    println!("寫入完成。WriteToken disk_id={}", token.disk_id);

    let verify = parse_gpt(&writable)?;
    for part in &draft.partitions {
        validate_alignment(part.start_lba, SectorSize::S512)?;
    }
    println!("驗證: 讀回 {} 個分區", verify.partitions.len());
    Ok(())
}

pub fn tui(image: Option<&str>) -> Result<()> {
    rspfdisk_tui::run(image)
}

fn resolve_template(name: &str) -> Result<rspfdisk_layouts::template::LayoutTemplate> {
    let aliases = [
        ("windows-standard", "windows_uefi_standard"),
        ("windows_uefi_standard", "windows_uefi_standard"),
        ("linux-ext4-home", "linux_ext4_home"),
        ("linux_ext4_home", "linux_ext4_home"),
        ("macos-apfs", "macos_apfs_target"),
        ("macos_apfs_target", "macos_apfs_target"),
    ];
    let resolved = aliases
        .iter()
        .find(|(a, _)| *a == name)
        .map(|(_, t)| *t)
        .unwrap_or(name);

    let template_path = Path::new("templates").join(format!("{resolved}.toml"));
    if template_path.exists() {
        return load_template(&template_path).map_err(|e| anyhow!("{e}"));
    }

    let alt = Path::new("../../templates").join(format!("{resolved}.toml"));
    if alt.exists() {
        return load_template(&alt).map_err(|e| anyhow!("{e}"));
    }

    let mut reg = TemplateRegistry::new();
    reg.load_dir("templates").ok();
    reg.load_dir("../../templates").ok();
    reg.get(resolved).cloned().map_err(|e| anyhow!("{e}"))
}

fn ensure_target_exists(path: &str) -> Result<()> {
    let p = Path::new(path);
    if p.exists() {
        return Ok(());
    }

    if classify_path(path) == DevicePathKind::LinuxBlockDevice {
        return Err(anyhow!("裝置不存在: {path}"));
    }

    create_test_image(path, 8 * 1024 * 1024 * 1024)?;
    println!("已建立測試 image: {path} (8 GiB)");
    Ok(())
}

fn parse_root_size(s: &str) -> Result<u64> {
    rspfdisk_layouts::size::parse_byte_size(s).map_err(|e| anyhow!("{e}"))
}
