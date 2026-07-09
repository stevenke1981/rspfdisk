mod support;

use std::process::Command;

use support::ensure_sparse_image;

fn rspfdisk() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rspfdisk"))
}

#[test]
fn layout_dry_run_windows() {
    let path = ensure_sparse_image("integration-dryrun.img");
    let path_str = path.to_str().unwrap();

    let output = rspfdisk()
        .args(["layout", "windows-standard", path_str, "--dry-run"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("EFI System"));
    assert!(stdout.contains("No write performed"));
}

#[test]
#[ignore = "slow 8GiB image write; covered by release gate with --include-ignored"]
fn layout_write_and_inspect() {
    let path = ensure_sparse_image("integration-write.img");
    let path_str = path.to_str().unwrap();

    let write_out = rspfdisk()
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
        write_out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&write_out.stderr)
    );

    let inspect_out = rspfdisk()
        .args(["inspect", path_str, "--json"])
        .output()
        .unwrap();
    assert!(inspect_out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&inspect_out.stdout).unwrap();
    let parts = json["partition_table"]["partitions"].as_array().unwrap();
    assert_eq!(parts.len(), 4);
}
