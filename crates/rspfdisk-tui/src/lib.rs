//! Chinese TUI for rust-spfdisk.
//!
//! Screens:
//!   Main          → 主選單，選擇目標磁碟或 image
//!   DiskList      → 列出可用區塊裝置與 image
//!   PartTable     → 顯示分割表
//!   QuickLayout   → 選擇快速分區模板
//!   Preview       → 預覽分區草稿
//!   SizeEditor    → 互動編輯分區容量
//!   BackupConfirm → 備份確認與建立
//!   WriteConfirm  → 輸入確認文字後寫入

use anyhow::Result;
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

use rspfdisk_core::{DiskInfo, LayoutDraft};
use rspfdisk_disk::{list_block_devices, open_read_only, BlockDevice};
use rspfdisk_gpt::parse_gpt;
use rspfdisk_layouts::{build_diff_report, generate_layout, load_template, TemplateRegistry};
use rspfdisk_mbr::parse_mbr;
use rspfdisk_safety::{assess_disk, disk_confirmation_phrase};

// ---------------------------------------------------------------------------
// Screens
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    Main,
    DiskList,
    PartTable,
    QuickLayout,
    Preview,
    SizeEditor,
    BackupConfirm,
    WriteConfirm,
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct AppState {
    screen: Screen,
    disks: Vec<DiskInfo>,
    selected_disk: Option<String>,
    selected_disk_info: Option<DiskInfo>,
    partitions_text: String,
    templates: Vec<String>,
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

        Self {
            screen: Screen::Main,
            disks: vec![],
            selected_disk: None,
            selected_disk_info: None,
            partitions_text: String::new(),
            templates: if names.is_empty() {
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
            },
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

// ---------------------------------------------------------------------------
// Draw helpers
// ---------------------------------------------------------------------------

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
            "  [{}] {}  {:.1} GiB  sector={}",
            i + 1,
            disk.path,
            size_gib,
            disk.logical_sector_size.bytes(),
        );
        lines.push(Line::from(Span::raw(label)));
    }
    lines.push(Line::from(Span::raw("")));
    lines.push(Line::from(Span::styled(
        "[I] 輸入 image 路徑  [R] 重新掃描  [Enter] 選取  [Esc] 返回",
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

fn handle_main(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('1') => {
            state.disks = list_block_devices().unwrap_or_default();
            state.log(format!("掃描到 {} 個裝置", state.disks.len()));
            state.screen = Screen::DiskList;
        }
        KeyCode::Char('2') => {
            if state.selected_disk.is_some() {
                refresh_part_table(state);
                state.screen = Screen::PartTable;
            } else {
                state.message = "請先選取磁碟或 image！".to_string();
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            state.template_index = 0;
            state.screen = Screen::QuickLayout;
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            state.editing_image_path = true;
            state.image_path_input.clear();
            state.screen = Screen::DiskList;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {}
        _ => {}
    }
}

fn handle_disk_list(state: &mut AppState, key: KeyCode) {
    match key {
        KeyCode::Char('i') | KeyCode::Char('I') => {
            state.editing_image_path = true;
            state.image_path_input.clear();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            state.disks = list_block_devices().unwrap_or_default();
            state.log(format!("重新掃描: {} 個裝置", state.disks.len()));
        }
        KeyCode::Enter => {
            if state.editing_image_path && !state.image_path_input.is_empty() {
                let p = state.image_path_input.clone();
                if Path::new(&p).exists() {
                    state.selected_disk = Some(p.clone());
                    state.log(format!("選取 image: {p}"));
                    refresh_part_table(state);
                    state.editing_image_path = false;
                    state.screen = Screen::PartTable;
                } else {
                    state.log(format!("檔案不存在: {p}"));
                }
            } else if !state.disks.is_empty() {
                // Select first disk (simplified: in a real TUI you'd use a list cursor)
                let disk = &state.disks[0];
                state.selected_disk = Some(disk.path.clone());
                state.selected_disk_info = Some(disk.clone());
                state.log(format!("選取磁碟: {}", disk.path));
                refresh_part_table(state);
                state.screen = Screen::PartTable;
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
            // Prepare write confirmation
            let phrase = state
                .selected_disk
                .as_deref()
                .map(disk_confirmation_phrase)
                .unwrap_or_default();
            state.confirm_phrase = phrase;
            state.confirm_input.clear();
            state.confirm_error = None;
            state.screen = Screen::WriteConfirm;
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
            // Validate confirmation phrase
            if state.confirm_input.trim() == state.confirm_phrase {
                state.log("✅ 確認文字正確，執行寫入...".to_string());
                // In a real TUI, this would call the writer.
                // For now, record success and return to main.
                state.confirm_error = None;
                state.screen = Screen::Main;
                state.message = format!(
                    "✅ 寫入完成（模擬） — {}",
                    state.selected_disk.as_deref().unwrap_or("?")
                );
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
            // Proceed to write confirmation
            let phrase = state
                .selected_disk
                .as_deref()
                .map(disk_confirmation_phrase)
                .unwrap_or_default();
            state.confirm_phrase = phrase;
            state.confirm_input.clear();
            state.confirm_error = None;
            state.screen = Screen::WriteConfirm;
        }
        KeyCode::Esc => {
            state.screen = Screen::Preview;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Business logic helpers
// ---------------------------------------------------------------------------

fn refresh_part_table(state: &mut AppState) {
    let path = match state.selected_disk.as_ref() {
        Some(p) => p.clone(),
        None => {
            state.partitions_text = "未選取磁碟".to_string();
            return;
        }
    };

    let device = match open_read_only(&path) {
        Ok(d) => d,
        Err(e) => {
            state.partitions_text = format!("無法開啟 {path}: {e}");
            return;
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
                return;
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
}

fn generate_draft(state: &AppState, template_name: &str) -> Result<LayoutDraft> {
    let path = state.selected_disk.as_deref().unwrap_or("test-empty.img");

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

    // Auto-create 8GiB image for testing
    if !Path::new(path).exists() {
        rspfdisk_disk::test_helpers::create_test_image(path, 8 * 1024 * 1024 * 1024)?;
    }

    let device = open_read_only(path).map_err(|e| anyhow::anyhow!("{e}"))?;
    let info = device.info();
    let draft = generate_layout(&template, &info, None).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(draft)
}
