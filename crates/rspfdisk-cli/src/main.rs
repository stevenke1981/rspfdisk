//! rspfdisk CLI — 磁碟分割工具
//!
//! Subcommands:
//!   list      List block devices
//!   inspect   Inspect partition table (MBR/GPT)
//!   backup    Backup partition table to `.rspbak`
//!   restore   Restore partition table (dry-run only)
//!   layout    Quick partition layout wizard
//!   tui       Launch terminal UI
//!
//! Safety: reads are default; writes require `--write` + confirmation.

mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rspfdisk", version, about = "Rust SPFDisk — 磁碟分割工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出可用磁碟/image
    List,
    /// 檢視分割表
    Inspect {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// 備份分割表
    Backup {
        path: String,
        #[arg(long)]
        out: String,
    },
    /// 還原預覽 (dry-run)
    Restore {
        path: String,
        backup: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// 套用快速分區模板
    Layout {
        template: String,
        path: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes_i_know_this_is_an_image: bool,
        /// 真實磁碟寫入確認文字（磁碟代號，例如 nvme0n1）
        #[arg(long)]
        confirm: Option<String>,
        /// 明確接受系統碟寫入風險
        #[arg(long)]
        accept_system_disk_risk: bool,
        #[arg(long)]
        root_size: Option<String>,
    },
    /// 啟動 TUI
    Tui {
        #[arg(long)]
        image: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::List => commands::list_disks(),
        Commands::Inspect { path, json } => commands::inspect(&path, json),
        Commands::Backup { path, out } => commands::backup(&path, &out),
        Commands::Restore {
            path,
            backup,
            dry_run,
        } => commands::restore(&path, &backup, dry_run),
        Commands::Layout {
            template,
            path,
            dry_run,
            write,
            yes_i_know_this_is_an_image,
            confirm,
            accept_system_disk_risk,
            root_size,
        } => commands::layout(&commands::LayoutOptions {
            template_name: &template,
            path: &path,
            dry_run,
            write,
            image_confirmed: yes_i_know_this_is_an_image,
            confirm_phrase: confirm.as_deref(),
            accept_system_disk_risk,
            root_size: root_size.as_deref(),
        }),
        Commands::Tui { image } => commands::tui(image.as_deref()),
    }
}
