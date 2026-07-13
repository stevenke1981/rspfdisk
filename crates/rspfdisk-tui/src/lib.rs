//! TUI (Terminal User Interface) for rspfdisk.
//!
//! Built with ratatui + crossterm. Supports Chinese (default) and English.
//!
//! ## Screens (9 total)
//!
//! | Screen         | Purpose                               |
//! |----------------|---------------------------------------|
//! | GuidedScenario | Choose Windows/Linux/macOS/multiboot  |
//! | Main           | Menu — select disk or image           |
//! | DiskList       | List block devices + enter image path |
//! | PartTable      | Display MBR/GPT partitions            |
//! | QuickLayout    | Select partition template             |
//! | Preview        | Preview partition draft diff          |
//! | SizeEditor     | Interactively edit partition sizes    |
//! | BackupConfirm  | Create backup before write            |
//! | WriteConfirm   | Type confirmation phrase to write     |
//!
//! ## Flow
//!
//! ```text
//! GuidedScenario → DiskList → QuickLayout → Preview
//!                                                 ↓
//!                                           SizeEditor
//!                                           BackupConfirm
//!                                           WriteConfirm
//! ```
//!
//! ## i18n
//! Language is selected via `RSPFDISK_LANG` env var (zh-TW default, en for English).

use anyhow::{anyhow, bail, Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;
use std::path::Path;

use rspfdisk_core::{ChangePlan, DiskInfo, LayoutDraft, PartitionTable};
use rspfdisk_disk::{
    classify_path, list_block_devices, open_read_only, open_read_write, BlockDevice, DevicePathKind,
};
use rspfdisk_gpt::{parse_gpt, write_gpt_from_draft};
use rspfdisk_layouts::{build_diff_report, generate_layout, load_template, TemplateRegistry};
use rspfdisk_mbr::parse_mbr;
use rspfdisk_safety::{assess_disk, confirm_write, disk_confirmation_phrase, ConfirmationOptions};

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    GuidedScenario,
    Main,
    DiskList,
    PartTable,
    QuickLayout,
    Preview,
    SizeEditor,
    BackupConfirm,
    WriteConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuidedScenario {
    Windows,
    Linux,
    Macos,
    Multiboot,
}

impl GuidedScenario {
    const ALL: [Self; 4] = [Self::Windows, Self::Linux, Self::Macos, Self::Multiboot];

    fn label(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Linux => "Linux",
            Self::Macos => "macOS",
            Self::Multiboot => "多重開機",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Windows => "Windows 10/11 UEFI 標準分區",
            Self::Linux => "Linux UEFI ext4 單系統",
            Self::Macos => "macOS GPT/APFS 目標分區（不格式化 APFS）",
            Self::Multiboot => "Windows + Linux GPT/UEFI 分區",
        }
    }

    fn template_name(self) -> &'static str {
        match self {
            Self::Windows => "windows_uefi_standard",
            Self::Linux => "linux_ext4_standard",
            Self::Macos => "macos_apfs_target",
            Self::Multiboot => "multiboot_windows_linux",
        }
    }
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct AppState {
    screen: Screen,
    disks: Vec<DiskInfo>,
    disk_index: usize,
    selected_disk: Option<String>,
    selected_disk_info: Option<DiskInfo>,
    partitions_text: String,
    templates: Vec<String>,
    guided_index: usize,
    selected_template: String,
    template_index: usize,
    draft: Option<LayoutDraft>,
    preview_text: String,
    confirm_input: String,
    confirm_phrase: String,
    confirm_error: Option<String>,
    message: String,
    image_path_input: String,
    editing_image_path: bool,
    log_lines: Vec<String>,
    // SizeEditor state
    editor_selected: usize,
    editor_input: String,
    editor_error: Option<String>,
    // BackupConfirm state
    backup_status: String,
    backup_path: Option<String>,
}

impl AppState {
    fn new() -> Self {
        let mut reg = TemplateRegistry::new();
        reg.load_dir("templates").ok();
        reg.load_dir("../../templates").ok();
        let names: Vec<String> = reg.names().into_iter().map(|s| s.to_string()).collect();

        let mut templates = if names.is_empty() {
            vec![
                "windows_uefi_standard".into(),
                "windows_uefi_with_data".into(),
                "windows_uefi_dual_boot".into(),
                "macos_apfs_target".into(),
                "macos_apfs_shared_exfat".into(),
                "macos_apfs_reserve_windows".into(),
                "linux_ext4_standard".into(),
                "linux_ext4_home".into(),
                "linux_btrfs_standard".into(),
                "linux_bios_gpt_biosboot".into(),
                "windows_legacy_mbr".into(),
            ]
        } else {
            names
        };

        for scenario in GuidedScenario::ALL {
            if !templates
                .iter()
                .any(|name| name == scenario.template_name())
            {
                templates.push(scenario.template_name().into());
            }
        }

        Self {
            screen: Screen::GuidedScenario,
            disks: vec![],
            disk_index: 0,
            selected_disk: None,
            selected_disk_info: None,
            partitions_text: String::new(),
            templates,
            guided_index: 0,
            selected_template: String::new(),
            template_index: 0,
            draft: None,
            preview_text: String::new(),
            confirm_input: String::new(),
            confirm_phrase: String::new(),
            confirm_error: None,
            message: String::new(),
            image_path_input: String::new(),
            editing_image_path: false,
            log_lines: vec![],
            editor_selected: 0,
            editor_input: String::new(),
            editor_error: None,
            backup_status: "尚未備份".to_string(),
            backup_path: None,
        }
    }

    fn log(&mut self, msg: String) {
        self.log_lines.push(msg);
        if self.log_lines.len() > 100 {
            self.log_lines.remove(0);
        }
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn run(image_path: Option<&str>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut ready_marker_emitted = false;

    let mut state = AppState::new();

    // Auto-select image if provided via CLI
    if let Some(img) = image_path {
        state.selected_disk = Some(img.to_string());
        state.log(format!("已指定 image: {img}"));
    }

    // Load templates on start
    state.log(format!("已載入 {} 個模板", state.templates.len()));

    'main: loop {
        // --- Draw ---
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(f.area());

            // Title bar
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    " Rust SPFDisk ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" v0.1.0  "),
                Span::styled(
                    state.selected_disk.as_deref().unwrap_or("（無磁碟）"),
                    Style::default().fg(Color::Yellow),
                ),
            ]))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);
            f.render_widget(title, chunks[0]);

            // Body
            let body_area = chunks[1];
            match state.screen {
                Screen::GuidedScenario => draw_guided_scenario(f, body_area, &state),
                Screen::Main => draw_main(f, body_area, &state),
                Screen::DiskList => draw_disk_list(f, body_area, &state),
                Screen::PartTable => draw_part_table(f, body_area, &state),
                Screen::QuickLayout => draw_quick_layout(f, body_area, &state),
                Screen::Preview => draw_preview(f, body_area, &state),
                Screen::SizeEditor => draw_size_editor(f, body_area, &state),
                Screen::BackupConfirm => draw_backup_confirm(f, body_area, &state),
                Screen::WriteConfirm => draw_write_confirm(f, body_area, &state),
            }

            // Footer
            draw_footer(f, chunks[2], &state);
        })?;

        if !ready_marker_emitted {
            emit_boot_ready_marker();
            ready_marker_emitted = true;
        }

        // --- Input ---
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                handle_key(&mut state, key.code);
                if state.screen == Screen::Main && key.code == KeyCode::Char('q') {
                    break 'main;
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn emit_boot_ready_marker() {
    use std::fs::OpenOptions;
    use std::io::Write;

    if let Ok(mut serial) = OpenOptions::new().write(true).open("/dev/ttyS0") {
        let _ = writeln!(serial, "RSPFDISK_TUI_READY");
    }
}

#[cfg(not(target_os = "linux"))]
fn emit_boot_ready_marker() {}

// ---------------------------------------------------------------------------
// Draw helpers
// ---------------------------------------------------------------------------

fn draw_guided_scenario(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let disk_label = state.selected_disk.as_deref().unwrap_or("（尚未選擇）");
    let mut lines = vec![
        Line::from(Span::styled(
            "第一次使用：選擇要準備的系統",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!("目標磁碟或 image: {disk_label}")),
        Line::from(Span::raw("")),
    ];

    for (index, scenario) in GuidedScenario::ALL.iter().enumerate() {
        let marker = if index == state.guided_index {
            " ▶"
        } else {
            "  "
        };
        let style = if index == state.guided_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{marker} [{}] {}", index + 1, scenario.label()),
                style,
            ),
            Span::raw(format!(" — {}", scenario.description())),
        ]));
    }

    lines.extend([
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[A] 進階模板列表",
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::styled(
            "[D] 選擇磁碟  [I] 輸入 image 路徑",
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::styled(
            "選擇後仍會先顯示草稿，寫入需要備份與明確確認。",
            Style::default().fg(Color::DarkGray),
        )),
    ]);

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("引導模式"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_main(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let disk_label = state.selected_disk.as_deref().unwrap_or("（無）");
    let items = vec![
        Line::from(format!("目標磁碟: {disk_label}")),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[1] 磁碟列表",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "[2] 檢視分割表",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "[F] 快速分區精靈",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::styled(
            "[G] 引導模式",
            Style::default().fg(Color::Green),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "[I] 輸入 image 路徑",
            Style::default().fg(Color::Blue),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled("[Q] 離開", Style::default().fg(Color::Red))),
    ];
    let para = Paragraph::new(Text::from(items))
        .block(Block::default().borders(Borders::ALL).title("主選單"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);

    // Message overlay
    if !state.message.is_empty() {
        let msg = Paragraph::new(state.message.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        let popup = centered_rect(60, 20, area);
        f.render_widget(msg, popup);
    }
}

fn draw_disk_list(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    if !state.message.is_empty() {
        lines.push(Line::from(Span::styled(
            state.message.as_str(),
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(Span::raw("")));
    }
    if state.disks.is_empty() {
        lines.push(Line::from(Span::styled(
            "⚠️ 未偵測到區塊裝置（Windows 上需指定 image）",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(Span::raw("")));
    }
    for (i, disk) in state.disks.iter().enumerate() {
        let size_gib = disk.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let label = format!(
            "{} [{}] {}  {:.1} GiB  sector={}",
            if i == state.disk_index { ">" } else { " " },
            i + 1,
            disk.path,
            size_gib,
            disk.logical_sector_size.bytes(),
        );
        let style = if i == state.disk_index {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(label, style)));
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "[↑↓] 選擇  [I] 輸入 image 路徑  [R] 重新掃描  [Enter] 選取  [Esc] 返回",
        Style::default().fg(Color::Blue),
    )));

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("磁碟列表"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_part_table(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let lines: Vec<Line> = if state.partitions_text.is_empty() {
        vec![Line::from(Span::styled(
            "請先選取磁碟（按 [1] 磁碟列表 或 [I] 輸入 image）",
            Style::default().fg(Color::Yellow),
        ))]
    } else {
        state
            .partitions_text
            .lines()
            .map(|l| {
                if l.starts_with("⚠") || l.starts_with("警告") {
                    Line::from(Span::styled(
                        l.to_string(),
                        Style::default().fg(Color::Yellow),
                    ))
                } else {
                    Line::from(Span::raw(l.to_string()))
                }
            })
            .collect()
    };

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("分割表"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);

    // Footer hint inside body for this screen
    let hint = Paragraph::new(Line::from(Span::styled(
        "[F] 快速分區  [Esc] 返回",
        Style::default().fg(Color::Blue),
    )))
    .alignment(Alignment::Right);
    let hint_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(1),
        area.width,
        1,
    );
    f.render_widget(hint, hint_area);
}

fn draw_quick_layout(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let mut lines: Vec<Line> = Vec::new();
    for (i, name) in state.templates.iter().enumerate() {
        let marker = if i == state.template_index {
            " ▶"
        } else {
            "  "
        };
        lines.push(Line::from(Span::raw(format!(
            "{marker} [{}] {name}",
            i + 1
        ))));
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "↑/↓ 選取  [Enter] 產生草稿  [Esc] 返回",
        Style::default().fg(Color::Blue),
    )));

    let list_w = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("快速分區精靈 — 選擇模板"),
        )
        .alignment(Alignment::Left);
    f.render_widget(list_w, area);
}

fn draw_preview(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let lines: Vec<Line> = state
        .preview_text
        .lines()
        .map(|l| Line::from(Span::raw(l.to_string())))
        .collect();

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("分區草稿預覽"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);

    let hint = Paragraph::new(Line::from(Span::styled(
        "[E] 編輯容量  [B] 備份  [W] 寫入確認  [Esc] 返回  [Q] 離開",
        Style::default().fg(Color::Blue),
    )))
    .alignment(Alignment::Right);
    let hint_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(1),
        area.width,
        1,
    );
    f.render_widget(hint, hint_area);
}

fn draw_write_confirm(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let confirm_phrase = state.confirm_phrase.clone();
    let input_display = if state.confirm_input.is_empty() {
        "（輸入確認文字）".to_string()
    } else {
        state.confirm_input.clone()
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "⚠️  寫入確認",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::raw(format!(
            "目標磁碟: {}",
            state.selected_disk.as_deref().unwrap_or("?")
        ))),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "請輸入確認文字以允許寫入：",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            format!("  確認文字: {confirm_phrase}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::raw(format!("> {input_display}"))),
        Line::from(Span::raw("")),
    ];

    if let Some(ref err) = state.confirm_error {
        lines.push(Line::from(Span::styled(
            format!("❌ {err}"),
            Style::default().fg(Color::Red),
        )));
    }

    lines.push(Line::from(Span::styled(
        "[Enter] 確認  [Esc] 取消",
        Style::default().fg(Color::Blue),
    )));

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("寫入確認"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_footer(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let help = match state.screen {
        Screen::GuidedScenario => {
            "↑↓:選擇 Enter:確認 1-4:直接選擇 A:進階模板 D:磁碟 I:image Esc:主選單"
        }
        Screen::Main => "Q:離開 1:磁碟列表 2:分割表 F:快速分區 I:輸入image",
        Screen::DiskList => "I:輸入image R:重新掃描 Enter:選取 Esc:返回",
        Screen::PartTable => "F:快速分區 Esc:返回",
        Screen::QuickLayout => "↑↓:選取 Enter:產生草稿 Esc:返回",
        Screen::Preview => "E:編輯容量 B:備份 W:寫入 Esc:返回 Q:離開",
        Screen::SizeEditor => "↑↓:選分區 Enter:編輯 Tab:切換欄位 Esc:返回",
        Screen::BackupConfirm => "B:建立備份 W:寫入 Esc:返回",
        Screen::WriteConfirm => "Enter:確認 Esc:取消",
    };
    let footer = Paragraph::new(help)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn draw_size_editor(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let draft = match &state.draft {
        Some(d) => d,
        None => {
            let para = Paragraph::new("無分區草稿，請先產生快速分區。")
                .block(Block::default().borders(Borders::ALL).title("容量編輯器"))
                .alignment(Alignment::Center);
            f.render_widget(para, area);
            return;
        }
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!("模板: {}", draft.display_name),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::raw("")));

    for (i, part) in draft.partitions.iter().enumerate() {
        let size_gib = part.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let size_display = if part.size_bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GiB", size_gib)
        } else {
            format!("{} MiB", part.size_bytes / (1024 * 1024))
        };
        let cursor = if i == state.editor_selected && state.editor_input.is_empty() {
            " ▶"
        } else {
            "  "
        };
        let line = format!(
            "{cursor} [{idx}] {name:20} {size:>10}  {fs:<6}",
            idx = i + 1,
            name = part.name,
            size = size_display,
            fs = part.filesystem.as_deref().unwrap_or("none"),
        );
        if i == state.editor_selected {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::Green),
            )));
            // Show input field if editing
            if !state.editor_input.is_empty() || state.editor_error.is_some() {
                let input_display = if state.editor_input.is_empty() {
                    "（輸入新大小，例如 80GiB）".to_string()
                } else {
                    format!("> {}", state.editor_input)
                };
                lines.push(Line::from(Span::styled(
                    format!("    大小: {input_display}"),
                    Style::default().fg(Color::Yellow),
                )));
                if let Some(ref err) = state.editor_error {
                    lines.push(Line::from(Span::styled(
                        format!("    ❌ {err}"),
                        Style::default().fg(Color::Red),
                    )));
                }
            }
        } else {
            lines.push(Line::from(Span::raw(line)));
        }
    }

    // Show total / remaining
    lines.push(Line::from(Span::raw("")));
    let total: u64 = draft.partitions.iter().map(|p| p.size_bytes).sum();
    let total_gib = total as f64 / (1024.0 * 1024.0 * 1024.0);
    lines.push(Line::from(Span::styled(
        format!("分區總計: {total_gib:.2} GiB"),
        Style::default().fg(Color::Blue),
    )));

    if let Some(ref info) = state.selected_disk_info {
        let disk_gib = info.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        let remaining = info.size_bytes.saturating_sub(total);
        let rem_gib = remaining as f64 / (1024.0 * 1024.0 * 1024.0);
        lines.push(Line::from(Span::styled(
            format!("磁碟容量: {disk_gib:.2} GiB  剩餘: {rem_gib:.2} GiB"),
            Style::default().fg(if remaining < 1024 * 1024 * 1024 {
                Color::Red
            } else {
                Color::Blue
            }),
        )));
    }

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("容量編輯器"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_backup_confirm(f: &mut ratatui::Frame, area: Rect, state: &AppState) {
    let mut lines = vec![
        Line::from(Span::styled(
            "💾  備份確認",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(Span::raw(format!(
            "目標磁碟: {}",
            state.selected_disk.as_deref().unwrap_or("?")
        ))),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("備份狀態: {}", state.backup_status),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::raw("")),
    ];

    if let Some(ref bp) = state.backup_path {
        lines.push(Line::from(Span::styled(
            format!("備份路徑: {bp}"),
            Style::default().fg(Color::Green),
        )));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "按 [W] 繼續寫入，或按 [B] 重新備份",
            Style::default().fg(Color::Blue),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "尚未建立備份。建議在寫入前建立備份。",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(Span::raw("")));
        lines.push(Line::from(Span::styled(
            "按 [B] 建立備份",
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "[Esc] 返回預覽",
        Style::default().fg(Color::Blue),
    )));

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("備份確認"))
        .alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height * (100 - percent_y)) / 200),
            Constraint::Length((r.height * percent_y) / 100),
            Constraint::Length((r.height * (100 - percent_y)) / 200),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width * (100 - percent_x)) / 200),
            Constraint::Length((r.width * percent_x) / 100),
            Constraint::Length((r.width * (100 - percent_x)) / 200),
        ])
        .split(popup_layout[1])[1]
}

// ---------------------------------------------------------------------------
// Key handling
// ---------------------------------------------------------------------------

fn handle_key(state: &mut AppState, key: KeyCode) {
    match state.screen {
        Screen::GuidedScenario => handle_guided_scenario(state, key),
        Screen::Main => handle_main(state, key),
        Screen::DiskList => handle_disk_list(state, key),
        Screen::PartTable => handle_part_table(state, key),
        Screen::QuickLayout => handle_quick_layout(state, key),
        Screen::Preview => handle_preview(state, key),
        Screen::SizeEditor => handle_size_editor(state, key),
        Screen::BackupConfirm => handle_backup_confirm(state, key),
        Screen::WriteConfirm => handle_write_confirm(state, key),
    }
}

fn handle_guided_scenario(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            state.guided_index = state.guided_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.guided_index = (state.guided_index + 1).min(GuidedScenario::ALL.len() - 1);
        }
        KeyCode::Enter => {
            let index = state.guided_index;
            select_guided_scenario(state, index);
        }
        KeyCode::Char(c) if ('1'..='4').contains(&c) => {
            select_guided_scenario(state, c as usize - '1' as usize);
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            state.selected_template.clear();
            state.template_index = 0;
            state.screen = Screen::QuickLayout;
        }
        KeyCode::Char('d') | KeyCode::Char('D') => open_disk_list(state),
        KeyCode::Char('i') | KeyCode::Char('I') => begin_image_input(state),
        KeyCode::Esc => state.screen = Screen::Main,
        _ => {}
    }
}

fn select_guided_scenario(state: &mut AppState, index: usize) {
    let Some(scenario) = GuidedScenario::ALL.get(index).copied() else {
        return;
    };

    let template_name = scenario.template_name();
    if !state.templates.iter().any(|name| name == template_name) {
        state.templates.push(template_name.to_string());
    }
    state.guided_index = index;
    state.template_index = state
        .templates
        .iter()
        .position(|name| name == template_name)
        .expect("guided template was inserted into the template list");
    state.selected_template = template_name.to_string();
    if state.selected_disk.is_some() {
        state.message.clear();
        state.screen = Screen::QuickLayout;
    } else {
        state.message = "請先選擇並檢查目標磁碟或 image，再套用配置。".to_string();
        open_disk_list(state);
    }
}

fn open_disk_list(state: &mut AppState) {
    state.disks = list_block_devices().unwrap_or_default();
    state.disk_index = state.disk_index.min(state.disks.len().saturating_sub(1));
    state.log(format!("掃描到 {} 個裝置", state.disks.len()));
    state.screen = Screen::DiskList;
}

fn begin_image_input(state: &mut AppState) {
    state.editing_image_path = true;
    state.image_path_input.clear();
    state.screen = Screen::DiskList;
}

fn handle_main(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('1') => open_disk_list(state),
        KeyCode::Char('2') => {
            if state.selected_disk.is_some() && refresh_part_table(state) {
                state.screen = Screen::PartTable;
            } else {
                state.message = "請先選取磁碟或 image！".to_string();
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.template_index = 0;
            state.screen = Screen::QuickLayout;
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            state.screen = Screen::GuidedScenario;
        }
        KeyCode::Char('i') | KeyCode::Char('I') => begin_image_input(state),
        KeyCode::Char('q') | KeyCode::Char('Q') => {}
        _ => {}
    }
}

fn handle_disk_list(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') if !state.editing_image_path => {
            state.disk_index = state.disk_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if !state.editing_image_path => {
            state.disk_index = (state.disk_index + 1).min(state.disks.len().saturating_sub(1));
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            state.editing_image_path = true;
            state.image_path_input.clear();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.disks = list_block_devices().unwrap_or_default();
            state.disk_index = state.disk_index.min(state.disks.len().saturating_sub(1));
            state.log(format!("重新掃描: {} 個裝置", state.disks.len()));
        }
        KeyCode::Enter => {
            if state.editing_image_path && !state.image_path_input.is_empty() {
                let p = state.image_path_input.clone();
                if Path::new(&p).exists() {
                    state.selected_disk = Some(p.clone());
                    state.log(format!("選取 image: {p}"));
                    if refresh_part_table(state) {
                        state.editing_image_path = false;
                        state.screen = next_screen_after_target_selection(state);
                    }
                } else {
                    state.log(format!("檔案不存在: {p}"));
                }
            } else if !state.disks.is_empty() {
                let disk = &state.disks[state.disk_index];
                state.selected_disk = Some(disk.path.clone());
                state.selected_disk_info = Some(disk.clone());
                state.log(format!("選取磁碟: {}", disk.path));
                if refresh_part_table(state) {
                    state.screen = next_screen_after_target_selection(state);
                }
            }
        }
        KeyCode::Esc => {
            state.editing_image_path = false;
            state.screen = Screen::Main;
        }
        KeyCode::Char(c) if state.editing_image_path => {
            if c == '\n' || c == '\r' {
                // handled by Enter above
            } else {
                state.image_path_input.push(c);
            }
        }
        KeyCode::Backspace if state.editing_image_path => {
            state.image_path_input.pop();
        }
        _ => {}
    }
}

fn next_screen_after_target_selection(state: &AppState) -> Screen {
    if state.selected_template.is_empty() {
        Screen::PartTable
    } else {
        Screen::QuickLayout
    }
}

fn handle_part_table(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.template_index = 0;
            state.screen = Screen::QuickLayout;
        }
        KeyCode::Esc => state.screen = Screen::Main,
        _ => {}
    }
}

fn handle_quick_layout(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            state.template_index = state.template_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.template_index = (state.template_index + 1).min(state.templates.len() - 1);
        }
        KeyCode::Enter => {
            if state.template_index < state.templates.len() {
                let name = state.templates[state.template_index].clone();
                state.selected_template = name.clone();
                match generate_draft(state, &name) {
                    Ok(draft) => {
                        state.draft = Some(draft.clone());
                        let diff =
                            build_diff_report(&rspfdisk_core::PartitionTable::empty(), &draft);
                        state.preview_text = diff.summary_lines.join("\n");
                        state.screen = Screen::Preview;
                    }
                    Err(e) => {
                        state.preview_text = format!("❌ 產生草稿失敗: {e}");
                        state.screen = Screen::Preview;
                    }
                }
            }
        }
        KeyCode::Esc => state.screen = Screen::Main,
        _ => {}
    }
}

fn handle_preview(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('e') | KeyCode::Char('E') => {
            if state.draft.is_some() {
                state.editor_selected = 0;
                state.editor_input.clear();
                state.editor_error = None;
                state.screen = Screen::SizeEditor;
            }
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            state.backup_status = "尚未備份".to_string();
            state.backup_path = None;
            state.screen = Screen::BackupConfirm;
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            if state
                .backup_path
                .as_deref()
                .is_some_and(|p| Path::new(p).is_file())
            {
                prepare_write_confirmation(state);
            } else {
                state.backup_status = "寫入前必須先建立備份".to_string();
                state.screen = Screen::BackupConfirm;
            }
        }
        KeyCode::Esc => state.screen = Screen::QuickLayout,
        KeyCode::Char('q') | KeyCode::Char('Q') => {}
        _ => {}
    }
}

fn handle_write_confirm(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char(c) if !c.is_control() => {
            state.confirm_input.push(c);
        }
        KeyCode::Backspace => {
            state.confirm_input.pop();
        }
        KeyCode::Enter => {
            if state.confirm_input.trim() == state.confirm_phrase {
                state.log("確認文字正確，開始寫入 image...".to_string());
                match write_confirmed_image(state) {
                    Ok(partition_count) => {
                        state.confirm_error = None;
                        state.screen = Screen::Main;
                        state.message = format!(
                            "寫入完成並驗證 {partition_count} 個分區 — {}",
                            state.selected_disk.as_deref().unwrap_or("?")
                        );
                        state.log(state.message.clone());
                    }
                    Err(error) => {
                        let message = format!("寫入失敗: {error:#}");
                        state.confirm_error = Some(message.clone());
                        state.log(message);
                    }
                }
            } else {
                state.confirm_error = Some(format!(
                    "輸入「{}」不正確，應為「{}」",
                    state.confirm_input.trim(),
                    state.confirm_phrase
                ));
            }
        }
        KeyCode::Esc => {
            state.confirm_input.clear();
            state.confirm_error = None;
            state.screen = Screen::Preview;
        }
        _ => {}
    }
}

fn handle_size_editor(state: &mut AppState, key: KeyCode) {
    let draft = match &state.draft {
        Some(d) => d.clone(),
        None => return,
    };

    match key {
        KeyCode::Up | KeyCode::Char('k') => {
            if state.editor_input.is_empty() {
                state.editor_selected = state.editor_selected.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if state.editor_input.is_empty() {
                let max = draft.partitions.len().saturating_sub(1);
                state.editor_selected = (state.editor_selected + 1).min(max);
            }
        }
        KeyCode::Enter => {
            if !state.editor_input.is_empty() {
                // Try to parse and apply the new size
                let expr = rspfdisk_layouts::size::parse_byte_size(&state.editor_input);
                match expr {
                    Ok(bytes) if bytes > 0 => {
                        // Update the draft partition size
                        let mut updated = draft.clone();
                        if let Some(part) = updated.partitions.get_mut(state.editor_selected) {
                            part.size_bytes = bytes;
                        }
                        // Recalculate start_lba for all partitions
                        let mut current_lba: u64 = 2048; // 1MiB aligned start
                        for part in updated.partitions.iter_mut() {
                            part.start_lba = current_lba;
                            let sectors = part.size_bytes.div_ceil(512);
                            current_lba = current_lba.checked_add(sectors).unwrap_or(current_lba);
                        }
                        // Update state
                        state.draft = Some(updated.clone());
                        let diff =
                            build_diff_report(&rspfdisk_core::PartitionTable::empty(), &updated);
                        state.preview_text = diff.summary_lines.join("\n");
                        state.editor_input.clear();
                        state.editor_error = None;
                        state.log(format!("分區 {} 大小已更新", state.editor_selected + 1));
                    }
                    Ok(_) => {
                        state.editor_error = Some("大小必須大於 0".to_string());
                    }
                    Err(e) => {
                        state.editor_error = Some(format!("大小格式錯誤: {e}"));
                    }
                }
            } else {
                // Enter when no input — start editing
                state.editor_input = "".to_string();
            }
        }
        KeyCode::Esc => {
            state.editor_input.clear();
            state.editor_error = None;
            state.screen = Screen::Preview;
        }
        KeyCode::Char(c) => {
            state.editor_input.push(c);
            state.editor_error = None; // Clear error on new input
        }
        KeyCode::Backspace => {
            state.editor_input.pop();
        }
        _ => {}
    }
}

fn handle_backup_confirm(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('b') | KeyCode::Char('B') => {
            let path = match state.selected_disk.as_ref() {
                Some(p) => p.clone(),
                None => {
                    state.backup_status = "錯誤: 未選取磁碟".to_string();
                    return;
                }
            };
            let backup_path = format!("{path}.rspbak");
            state.backup_status = "備份中...".to_string();
            state.log("正在建立備份...".to_string());

            let device = match rspfdisk_disk::open_read_only(&path) {
                Ok(d) => d,
                Err(e) => {
                    state.backup_status = format!("❌ 開啟磁碟失敗: {e}");
                    state.log(state.backup_status.clone());
                    return;
                }
            };

            match rspfdisk_backup::create_backup(&device, &backup_path) {
                Ok(manifest) => {
                    state.backup_path = Some(backup_path);
                    state.backup_status = format!(
                        "✅ 備份完成（{}，{}）",
                        manifest.partition_table,
                        manifest.created_at.format("%H:%M:%S")
                    );
                    state.log(state.backup_status.clone());
                }
                Err(e) => {
                    state.backup_status = format!("❌ 備份失敗: {e}");
                    state.log(state.backup_status.clone());
                }
            }
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            if state
                .backup_path
                .as_deref()
                .is_some_and(|p| Path::new(p).is_file())
            {
                prepare_write_confirmation(state);
            } else {
                state.backup_status = "寫入前必須先建立有效備份".to_string();
            }
        }
        KeyCode::Esc => {
            state.screen = Screen::Preview;
        }
        _ => {}
    }
}

fn prepare_write_confirmation(state: &mut AppState) {
    state.confirm_phrase = state
        .selected_disk
        .as_deref()
        .map(disk_confirmation_phrase)
        .unwrap_or_default();
    state.confirm_input.clear();
    state.confirm_error = None;
    state.screen = Screen::WriteConfirm;
}

fn write_confirmed_image(state: &AppState) -> Result<usize> {
    let path = state.selected_disk.as_deref().context("未選取目標 image")?;
    let target = Path::new(path);
    if classify_path(path) != DevicePathKind::ImageFile || !target.is_file() {
        bail!("TUI 僅允許寫入一般 image 檔案，不支援實體磁碟");
    }

    let backup_path = state.backup_path.as_deref().context("尚未建立備份")?;
    if !Path::new(backup_path).is_file() {
        bail!("備份檔不存在: {backup_path}");
    }

    let draft = state.draft.as_ref().context("沒有可寫入的分割區草稿")?;
    let read_only = open_read_only(path).map_err(|error| anyhow!("開啟 image 失敗: {error}"))?;
    let disk_info = read_only.info();
    let current = parse_gpt(&read_only).unwrap_or_else(|_| PartitionTable::empty());
    let diff = build_diff_report(&current, draft);
    let plan = ChangePlan {
        disk_path: path.to_string(),
        layout: draft.clone(),
        diff,
        backup_path: Some(backup_path.to_string()),
        dry_run: false,
    };

    let _token = confirm_write(
        &plan,
        &ConfirmationOptions {
            write: true,
            dry_run: false,
            image_confirmed: true,
            confirmation_phrase: Some(state.confirm_input.trim().to_string()),
            backup_path: Some(backup_path.to_string()),
            accept_system_disk_risk: false,
        },
        &disk_info,
    )
    .map_err(|error| anyhow!("安全確認失敗: {error}"))?;

    drop(read_only);
    let mut writable =
        open_read_write(path).map_err(|error| anyhow!("開啟 image 寫入失敗: {error}"))?;
    write_gpt_from_draft(&mut writable, draft).context("寫入 GPT")?;
    let verified = parse_gpt(&writable).context("寫入後讀回 GPT")?;

    if verified.partitions.len() != draft.partitions.len() {
        bail!(
            "讀回分區數量不符: 預期 {}，實際 {}",
            draft.partitions.len(),
            verified.partitions.len()
        );
    }
    for (expected, actual) in draft.partitions.iter().zip(&verified.partitions) {
        if actual.start_lba != expected.start_lba || actual.size_bytes != expected.size_bytes {
            bail!("讀回分區「{}」的位置或大小不符", expected.name);
        }
    }

    Ok(verified.partitions.len())
}

// ---------------------------------------------------------------------------
// Business logic helpers
// ---------------------------------------------------------------------------

fn refresh_part_table(state: &mut AppState) -> bool {
    let path = match state.selected_disk.as_ref() {
        Some(p) => p.clone(),
        None => {
            state.partitions_text = "未選取磁碟".to_string();
            state.message = state.partitions_text.clone();
            return false;
        }
    };

    let device = match open_read_only(&path) {
        Ok(d) => d,
        Err(e) => {
            state.partitions_text = format!("無法開啟 {path}: {e}");
            state.message = state.partitions_text.clone();
            return false;
        }
    };

    let info = device.info();
    state.selected_disk_info = Some(info.clone());

    // Try GPT, fallback to MBR
    let table = match parse_gpt(&device) {
        Ok(gpt) => gpt,
        Err(_) => match parse_mbr(&device) {
            Ok(mbr) => mbr,
            Err(e) => {
                state.partitions_text = format!("無法解析分割表: {e}");
                state.message = state.partitions_text.clone();
                return false;
            }
        },
    };

    let risk = assess_disk(&path, &info);
    let mut text = String::new();
    text.push_str(&format!("磁碟: {}\n", path));
    text.push_str(&format!(
        "容量: {:.2} GiB   磁區大小: {}\n",
        info.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        info.logical_sector_size.bytes(),
    ));
    text.push_str(&format!("分割表類型: {:?}\n", table.kind));
    text.push_str(&format!("開機模式: {:?}\n", table.boot_mode));
    text.push_str(&format!("風險等級: {:?}\n", risk.level));
    for w in &risk.warnings {
        text.push_str(&format!("  警告: {w}\n"));
    }
    text.push('\n');

    for part in &table.partitions {
        let size_gib = part.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        text.push_str(&format!(
            "[{}] {}  {:.2} GiB  {:?}  LBA {}-{}\n",
            part.index + 1,
            part.name,
            size_gib,
            part.partition_type,
            part.start_lba,
            part.end_lba,
        ));
    }
    for w in &table.warnings {
        text.push_str(&format!("⚠️  分割表警告: {w}\n"));
    }

    state.partitions_text = text;
    state.message.clear();
    true
}

fn generate_draft(state: &AppState, template_name: &str) -> Result<LayoutDraft> {
    let path = state
        .selected_disk
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("請先選擇目標磁碟或 image"))?;

    // Try loading template from file, then from registry
    let template_path = Path::new("templates").join(format!("{template_name}.toml"));
    let template = if template_path.exists() {
        load_template(&template_path).map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        let alt = Path::new("../../templates").join(format!("{template_name}.toml"));
        if alt.exists() {
            load_template(&alt).map_err(|e| anyhow::anyhow!("{e}"))?
        } else {
            let mut reg = TemplateRegistry::new();
            reg.load_dir("templates").ok();
            reg.load_dir("../../templates").ok();
            reg.get(template_name)
                .map_err(|e| anyhow::anyhow!("{e}"))?
                .clone()
        }
    };

    if !Path::new(path).exists() {
        anyhow::bail!("目標不存在: {path}");
    }

    let device = open_read_only(path).map_err(|e| anyhow::anyhow!("{e}"))?;
    let info = device.info();
    let draft = generate_layout(&template, &info, None).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(draft)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rspfdisk_core::{BootMode, PartitionDraft, PartitionTableKind, PartitionType, SectorSize};

    /// Helper: create an AppState with a known template list and draft.
    fn test_state() -> AppState {
        AppState {
            screen: Screen::Main,
            disks: vec![],
            disk_index: 0,
            selected_disk: Some("/dev/test".into()),
            selected_disk_info: None,
            partitions_text: String::new(),
            templates: vec![
                "windows_uefi_standard".into(),
                "windows_uefi_dual_boot".into(),
                "linux_ext4_standard".into(),
            ],
            guided_index: 0,
            selected_template: String::new(),
            template_index: 0,
            draft: None,
            preview_text: String::new(),
            confirm_input: String::new(),
            confirm_phrase: String::new(),
            confirm_error: None,
            message: String::new(),
            image_path_input: String::new(),
            editing_image_path: false,
            log_lines: Vec::new(),
            editor_selected: 0,
            editor_input: String::new(),
            editor_error: None,
            backup_status: String::new(),
            backup_path: None,
        }
    }

    fn test_disk(path: &str) -> DiskInfo {
        DiskInfo {
            path: path.into(),
            size_bytes: 64 * 1024 * 1024,
            logical_sector_size: SectorSize::S512,
            physical_sector_size: Some(SectorSize::S512),
            model: None,
            serial: None,
            removable: true,
            read_only: false,
        }
    }

    /// Helper: create a simple LayoutDraft for SizeEditor tests.
    fn test_draft() -> LayoutDraft {
        LayoutDraft {
            template_name: "test".into(),
            display_name: "Test".into(),
            table: PartitionTableKind::Gpt,
            boot_mode: BootMode::Uefi,
            partitions: vec![
                PartitionDraft {
                    name: "EFI System".into(),
                    start_lba: 2048,
                    size_bytes: 512 * 1024 * 1024,
                    partition_type: PartitionType::Esp,
                    filesystem: Some("fat32".into()),
                    mount_point: None,
                    note: None,
                    flags: vec![],
                },
                PartitionDraft {
                    name: "Linux Root".into(),
                    start_lba: 2064,
                    size_bytes: 80 * 1024 * 1024 * 1024,
                    partition_type: PartitionType::LinuxFilesystem,
                    filesystem: Some("ext4".into()),
                    mount_point: Some("/".into()),
                    note: None,
                    flags: vec![],
                },
                PartitionDraft {
                    name: "Linux Swap".into(),
                    start_lba: 2064 + (80 * 1024 * 1024 * 1024 / 512),
                    size_bytes: 8 * 1024 * 1024 * 1024,
                    partition_type: PartitionType::LinuxSwap,
                    filesystem: Some("swap".into()),
                    mount_point: None,
                    note: None,
                    flags: vec![],
                },
            ],
        }
    }

    // -----------------------------------------------------------------------
    // Screen navigation
    // -----------------------------------------------------------------------

    #[test]
    fn new_app_starts_in_guided_scenario() {
        let state = AppState::new();

        assert_eq!(state.screen, Screen::GuidedScenario);
        assert_eq!(state.guided_index, 0);
        assert!(state
            .templates
            .iter()
            .any(|name| name == "multiboot_windows_linux"));
    }

    #[test]
    fn disk_list_cursor_selects_the_highlighted_disk() {
        let mut state = test_state();
        state.screen = Screen::DiskList;
        state.selected_disk = None;
        state.disks = vec![test_disk("first.img"), test_disk("second.img")];

        handle_disk_list(&mut state, KeyCode::Down);
        assert_eq!(state.disk_index, 1);

        handle_disk_list(&mut state, KeyCode::Enter);
        assert_eq!(state.selected_disk.as_deref(), Some("second.img"));
    }

    #[test]
    fn disk_list_cursor_stays_within_bounds() {
        let mut state = test_state();
        state.screen = Screen::DiskList;
        state.disks = vec![test_disk("only.img")];

        handle_disk_list(&mut state, KeyCode::Up);
        handle_disk_list(&mut state, KeyCode::Down);

        assert_eq!(state.disk_index, 0);
    }

    #[test]
    fn disk_list_blocks_guided_flow_when_inspection_fails() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let image_path = std::env::temp_dir().join(format!("rspfdisk-malformed-{id}.img"));
        std::fs::write(&image_path, vec![0xff; 4096]).unwrap();

        let mut state = test_state();
        state.screen = Screen::DiskList;
        state.selected_disk = None;
        state.selected_template = "windows_uefi_standard".into();
        state.editing_image_path = true;
        state.image_path_input = image_path.to_string_lossy().into_owned();

        handle_disk_list(&mut state, KeyCode::Enter);

        assert_eq!(state.screen, Screen::DiskList);
        assert!(state.message.contains("無法解析分割表"));
        std::fs::remove_file(image_path).ok();
    }

    #[test]
    fn guided_scenarios_map_to_stable_template_names() {
        assert_eq!(
            GuidedScenario::Windows.template_name(),
            "windows_uefi_standard"
        );
        assert_eq!(GuidedScenario::Linux.template_name(), "linux_ext4_standard");
        assert_eq!(GuidedScenario::Macos.template_name(), "macos_apfs_target");
        assert_eq!(
            GuidedScenario::Multiboot.template_name(),
            "multiboot_windows_linux"
        );
    }

    #[test]
    fn guided_scenario_down_and_enter_select_multiboot_template() {
        let mut state = test_state();
        state.screen = Screen::GuidedScenario;

        for _ in 0..3 {
            handle_guided_scenario(&mut state, KeyCode::Down);
        }
        assert_eq!(state.guided_index, 3);

        handle_guided_scenario(&mut state, KeyCode::Enter);

        assert_eq!(state.screen, Screen::QuickLayout);
        assert_eq!(state.selected_template, "multiboot_windows_linux");
        assert_eq!(
            state.templates[state.template_index],
            "multiboot_windows_linux"
        );
    }

    #[test]
    fn guided_scenario_number_selects_windows_template() {
        let mut state = test_state();
        state.screen = Screen::GuidedScenario;

        handle_guided_scenario(&mut state, KeyCode::Char('1'));

        assert_eq!(state.screen, Screen::QuickLayout);
        assert_eq!(state.selected_template, "windows_uefi_standard");
        assert_eq!(state.template_index, 0);
    }

    #[test]
    fn guided_scenario_requires_target_before_layout() {
        let mut state = test_state();
        state.screen = Screen::GuidedScenario;
        state.selected_disk = None;

        handle_guided_scenario(&mut state, KeyCode::Char('1'));

        assert_eq!(state.screen, Screen::DiskList);
        assert_eq!(state.selected_template, "windows_uefi_standard");
        assert!(state.message.contains("請先選擇"));
    }

    #[test]
    fn guided_target_selection_continues_to_selected_layout() {
        let mut state = test_state();
        state.selected_template = "linux_ext4_standard".into();

        assert_eq!(
            next_screen_after_target_selection(&state),
            Screen::QuickLayout
        );

        state.selected_template.clear();
        assert_eq!(
            next_screen_after_target_selection(&state),
            Screen::PartTable
        );
    }

    #[test]
    fn guided_scenario_a_opens_advanced_template_list() {
        let mut state = test_state();
        state.screen = Screen::GuidedScenario;
        state.guided_index = 2;

        handle_guided_scenario(&mut state, KeyCode::Char('a'));

        assert_eq!(state.screen, Screen::QuickLayout);
        assert_eq!(state.template_index, 0);
        assert!(state.selected_template.is_empty());
    }

    #[test]
    fn main_screen_f_key_goes_to_quick_layout() {
        let mut state = test_state();
        handle_main(&mut state, KeyCode::Char('f'));
        assert_eq!(state.screen, Screen::QuickLayout);
        assert_eq!(state.template_index, 0);
    }

    #[test]
    fn main_screen_upper_f_goes_to_quick_layout() {
        let mut state = test_state();
        handle_main(&mut state, KeyCode::Char('F'));
        assert_eq!(state.screen, Screen::QuickLayout);
    }

    #[test]
    fn main_screen_q_does_not_change_screen() {
        let mut state = test_state();
        state.screen = Screen::Main;
        handle_main(&mut state, KeyCode::Char('q'));
        assert_eq!(state.screen, Screen::Main);
    }

    #[test]
    fn main_screen_2_without_selection_shows_message() {
        let mut state = test_state();
        state.selected_disk = None;
        handle_main(&mut state, KeyCode::Char('2'));
        assert_eq!(state.screen, Screen::Main);
        assert!(!state.message.is_empty());
    }

    // -----------------------------------------------------------------------
    // QuickLayout
    // -----------------------------------------------------------------------

    #[test]
    fn quick_layout_down_selects_next_template() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        assert_eq!(state.template_index, 0);
        handle_quick_layout(&mut state, KeyCode::Down);
        assert_eq!(state.template_index, 1);
        handle_quick_layout(&mut state, KeyCode::Down);
        assert_eq!(state.template_index, 2);
    }

    #[test]
    fn quick_layout_up_does_not_go_below_zero() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        handle_quick_layout(&mut state, KeyCode::Up);
        assert_eq!(state.template_index, 0);
    }

    #[test]
    fn quick_layout_down_stays_within_bounds() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        state.template_index = state.templates.len() - 1;
        handle_quick_layout(&mut state, KeyCode::Down);
        assert_eq!(state.template_index, state.templates.len() - 1);
    }

    #[test]
    fn quick_layout_jk_keys_navigate() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        handle_quick_layout(&mut state, KeyCode::Char('j'));
        assert_eq!(state.template_index, 1);
        handle_quick_layout(&mut state, KeyCode::Char('k'));
        assert_eq!(state.template_index, 0);
    }

    #[test]
    fn quick_layout_esc_returns_to_main() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        handle_quick_layout(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::Main);
    }

    // -----------------------------------------------------------------------
    // Preview → SizeEditor → WriteConfirm navigation
    // -----------------------------------------------------------------------

    #[test]
    fn preview_e_key_goes_to_size_editor() {
        let mut state = test_state();
        state.screen = Screen::Preview;
        state.draft = Some(test_draft());
        handle_preview(&mut state, KeyCode::Char('e'));
        assert_eq!(state.screen, Screen::SizeEditor);
        assert_eq!(state.editor_selected, 0);
    }

    #[test]
    fn preview_e_key_no_draft_stays() {
        let mut state = test_state();
        state.screen = Screen::Preview;
        state.draft = None;
        handle_preview(&mut state, KeyCode::Char('e'));
        assert_eq!(state.screen, Screen::Preview);
    }

    #[test]
    fn preview_b_key_goes_to_backup_confirm() {
        let mut state = test_state();
        state.screen = Screen::Preview;
        handle_preview(&mut state, KeyCode::Char('b'));
        assert_eq!(state.screen, Screen::BackupConfirm);
        assert_eq!(state.backup_status, "尚未備份");
    }

    #[test]
    fn preview_w_key_requires_backup_first() {
        let mut state = test_state();
        state.screen = Screen::Preview;
        handle_preview(&mut state, KeyCode::Char('w'));
        assert_eq!(state.screen, Screen::BackupConfirm);
        assert!(state.backup_status.contains("必須先建立備份"));
    }

    #[test]
    fn preview_esc_returns_to_quick_layout() {
        let mut state = test_state();
        state.screen = Screen::Preview;
        handle_preview(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::QuickLayout);
    }

    // -----------------------------------------------------------------------
    // SizeEditor
    // -----------------------------------------------------------------------

    #[test]
    fn size_editor_up_down_selects_partition() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        // Start at partition 0
        assert_eq!(state.editor_selected, 0);
        // Move down
        handle_size_editor(&mut state, KeyCode::Down);
        assert_eq!(state.editor_selected, 1);
        // Move down again
        handle_size_editor(&mut state, KeyCode::Down);
        assert_eq!(state.editor_selected, 2);
        // Move up
        handle_size_editor(&mut state, KeyCode::Up);
        assert_eq!(state.editor_selected, 1);
    }

    #[test]
    fn size_editor_up_stays_within_bounds() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        handle_size_editor(&mut state, KeyCode::Up);
        assert_eq!(state.editor_selected, 0);
    }

    #[test]
    fn size_editor_down_stays_within_bounds() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        // For 3 partitions, max index is 2
        state.editor_selected = 2;
        handle_size_editor(&mut state, KeyCode::Down);
        assert_eq!(state.editor_selected, 2);
    }

    #[test]
    fn size_editor_esc_returns_to_preview() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        handle_size_editor(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::Preview);
    }

    #[test]
    fn size_editor_applies_new_size() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        // Type a new size
        for c in "100GiB".chars() {
            handle_size_editor(&mut state, KeyCode::Char(c));
        }
        assert_eq!(state.editor_input, "100GiB");
        // Apply
        handle_size_editor(&mut state, KeyCode::Enter);
        // After applying, input should be cleared and draft updated
        assert!(state.editor_input.is_empty());
        assert!(state.draft.is_some());
        // The first partition should now be 100 GiB
        if let Some(draft) = &state.draft {
            assert_eq!(draft.partitions[0].size_bytes, 100 * 1024 * 1024 * 1024);
        }
    }

    #[test]
    fn size_editor_invalid_size_shows_error() {
        let mut state = test_state();
        state.screen = Screen::SizeEditor;
        state.draft = Some(test_draft());
        // Type invalid size
        for c in "abc".chars() {
            handle_size_editor(&mut state, KeyCode::Char(c));
        }
        handle_size_editor(&mut state, KeyCode::Enter);
        assert!(state.editor_error.is_some());
    }

    // -----------------------------------------------------------------------
    // WriteConfirm
    // -----------------------------------------------------------------------

    #[test]
    fn write_confirm_correct_phrase_without_image_is_rejected() {
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        state.confirm_phrase = "test-phrase".into();
        for c in "test-phrase".chars() {
            handle_write_confirm(&mut state, KeyCode::Char(c));
        }
        handle_write_confirm(&mut state, KeyCode::Enter);
        assert_eq!(state.screen, Screen::WriteConfirm);
        assert!(state
            .confirm_error
            .as_deref()
            .is_some_and(|error| error.contains("僅允許寫入一般 image")));
    }

    #[test]
    fn write_confirm_writes_and_verifies_image() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let image_path = std::env::temp_dir().join(format!("rspfdisk-tui-{id}.img"));
        let backup_path = std::env::temp_dir().join(format!("rspfdisk-tui-{id}.rspbak"));
        let device = rspfdisk_disk::create_test_image(&image_path, 64 * 1024 * 1024)
            .expect("create test image");
        rspfdisk_backup::create_backup(&device, &backup_path).expect("create image backup");
        drop(device);

        let image = image_path.to_string_lossy().into_owned();
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        state.selected_disk = Some(image.clone());
        state.draft = Some(LayoutDraft {
            template_name: "tui-write-test".into(),
            display_name: "TUI write test".into(),
            table: PartitionTableKind::Gpt,
            boot_mode: BootMode::Uefi,
            partitions: vec![PartitionDraft {
                name: "Data".into(),
                start_lba: 2048,
                size_bytes: 8 * 1024 * 1024,
                partition_type: PartitionType::MicrosoftBasicData,
                filesystem: None,
                mount_point: None,
                note: None,
                flags: vec![],
            }],
        });
        state.backup_path = Some(backup_path.to_string_lossy().into_owned());
        state.confirm_phrase = disk_confirmation_phrase(&image);
        for c in state.confirm_phrase.clone().chars() {
            handle_write_confirm(&mut state, KeyCode::Char(c));
        }

        handle_write_confirm(&mut state, KeyCode::Enter);

        assert_eq!(state.screen, Screen::Main);
        assert!(state.confirm_error.is_none());
        assert!(state.message.contains("寫入完成並驗證 1 個分區"));
        let verified = rspfdisk_disk::open_read_only(&image_path).expect("reopen test image");
        let table = parse_gpt(&verified).expect("parse written GPT");
        assert_eq!(table.partitions.len(), 1);
        assert_eq!(table.partitions[0].start_lba, 2048);

        std::fs::remove_file(&image_path).expect("remove test image");
        std::fs::remove_file(&backup_path).expect("remove test backup");
    }

    #[test]
    fn write_confirm_wrong_phrase_shows_error() {
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        state.confirm_phrase = "correct-phrase".into();
        // Type wrong phrase
        for c in "wrong".chars() {
            handle_write_confirm(&mut state, KeyCode::Char(c));
        }
        handle_write_confirm(&mut state, KeyCode::Enter);
        assert_eq!(state.screen, Screen::WriteConfirm);
        assert!(state.confirm_error.is_some());
    }

    #[test]
    fn write_confirm_empty_phrase_fails() {
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        state.confirm_phrase = "required-phrase".into();
        // Don't type anything, just press Enter
        handle_write_confirm(&mut state, KeyCode::Enter);
        assert_eq!(state.screen, Screen::WriteConfirm);
        assert!(state.confirm_error.is_some());
    }

    #[test]
    fn write_confirm_backspace_works() {
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        state.confirm_phrase = "ab".into();
        // Type "abc"
        for c in "abc".chars() {
            handle_write_confirm(&mut state, KeyCode::Char(c));
        }
        // Backspace twice → "a"
        handle_write_confirm(&mut state, KeyCode::Backspace);
        handle_write_confirm(&mut state, KeyCode::Backspace);
        assert_eq!(state.confirm_input, "a");
    }

    #[test]
    fn write_confirm_esc_returns_to_preview() {
        let mut state = test_state();
        state.screen = Screen::WriteConfirm;
        handle_write_confirm(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::Preview);
        assert!(state.confirm_input.is_empty());
    }

    // -----------------------------------------------------------------------
    // BackupConfirm (pure state transitions)
    // -----------------------------------------------------------------------

    #[test]
    fn backup_confirm_b_without_selection_shows_error() {
        let mut state = test_state();
        state.screen = Screen::BackupConfirm;
        state.selected_disk = None;
        handle_backup_confirm(&mut state, KeyCode::Char('b'));
        assert!(!state.backup_status.is_empty());
    }

    #[test]
    fn backup_confirm_w_without_backup_stays_on_screen() {
        let mut state = test_state();
        state.screen = Screen::BackupConfirm;
        handle_backup_confirm(&mut state, KeyCode::Char('w'));
        assert_eq!(state.screen, Screen::BackupConfirm);
        assert!(state.backup_status.contains("有效備份"));
    }

    // -----------------------------------------------------------------------
    // PartTable
    // -----------------------------------------------------------------------

    #[test]
    fn part_table_f_goes_to_quick_layout() {
        let mut state = test_state();
        state.screen = Screen::PartTable;
        handle_part_table(&mut state, KeyCode::Char('f'));
        assert_eq!(state.screen, Screen::QuickLayout);
    }

    #[test]
    fn part_table_esc_returns_to_main() {
        let mut state = test_state();
        state.screen = Screen::PartTable;
        handle_part_table(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::Main);
    }

    // -----------------------------------------------------------------------
    // handle_key dispatching
    // -----------------------------------------------------------------------

    #[test]
    fn handle_key_dispatches_to_current_screen() {
        let mut state = test_state();
        state.screen = Screen::QuickLayout;
        handle_key(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, Screen::Main);

        // Now from Main, go to QuickLayout
        handle_key(&mut state, KeyCode::Char('f'));
        assert_eq!(state.screen, Screen::QuickLayout);
    }
}
