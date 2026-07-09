//! Golden expectations for Windows UEFI standard template (test.md).

mod support;

use std::process::Command;

use support::{ensure_sparse_image, workspace_image, IMAGE_SIZE};

fn rspfdisk() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rspfdisk"))
}

/// Fast: layout engine only (no GPT write).
#[test]
fn golden_windows_standard_dry_run() {
    let path = ensure_sparse_image("golden-dryrun.img");
    let path_str = path.to_str().unwrap();

    let output = rspfdisk()
        .args(["layout", "windows-standard", path_str, "--dry-run"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EFI System"));
    assert!(stdout.contains("Microsoft Reserved"));
    assert!(stdout.contains("Windows Recovery"));
    assert!(stdout.contains("No write performed"));
}

/// Slow: full 8GiB GPT write + inspect (~2–3 min on Windows). Run with `--include-ignored`.
#[test]
#[ignore = "slow 8GiB image write; covered by release gate with --include-ignored"]
fn golden_windows_standard_layout() {
    let path = ensure_sparse_image("golden-windows-standard.img");
    let path_str = path.to_str().unwrap();

    let write = rspfdisk()
        .args([
            "layout",
            "windows-standard",
            path_str,
            "--write",
            "--yes-i-know-this-is-an-image",
        ])
        .output()
        .unwrap();
    assert!(
        write.status.success(),
        "{}",
        String::from_utf8_lossy(&write.stderr)
    );

    let inspect = rspfdisk()
        .args(["inspect", path_str, "--json"])
        .output()
        .unwrap();
    assert!(inspect.status.success());
    let json: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    let parts = json["partition_table"]["partitions"].as_array().unwrap();

    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0]["name"], "EFI System");
    assert_eq!(parts[1]["name"], "Microsoft Reserved");
    assert_eq!(parts[2]["name"], "Windows");
    assert_eq!(parts[3]["name"], "Windows Recovery");

    let esp_start = parts[0]["start_lba"].as_u64().unwrap();
    assert_eq!(esp_start % 2048, 0);
    assert!(esp_start >= 2048);

    assert_eq!(parts[0]["size_bytes"].as_u64().unwrap(), 512 * 1024 * 1024);
    assert_eq!(parts[1]["size_bytes"].as_u64().unwrap(), 16 * 1024 * 1024);
    assert_eq!(parts[3]["size_bytes"].as_u64().unwrap(), 1024 * 1024 * 1024);

    for i in 0..parts.len() {
        for j in (i + 1)..parts.len() {
            let a0 = parts[i]["start_lba"].as_u64().unwrap();
            let a1 = parts[i]["end_lba"].as_u64().unwrap();
            let b0 = parts[j]["start_lba"].as_u64().unwrap();
            let b1 = parts[j]["end_lba"].as_u64().unwrap();
            let overlap = a0 <= b1 && b0 <= a1;
            assert!(!overlap, "partition {i} overlaps {j}");
        }
    }
}

#[test]
fn golden_image_fixture_exists_or_documented() {
    let golden = workspace_image("golden-windows-standard.img");
    if golden.exists() {
        assert!(golden.metadata().unwrap().len() >= IMAGE_SIZE);
    }
}
