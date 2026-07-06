//! Chinese TUI for rust-spfdisk (smoke-test stub).

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    QuickLayout,
    Preview,
}

pub fn run(image: Option<&str>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut screen = Screen::Main;
    let mut selected_template = "未選擇";
    let disk_label = image.unwrap_or("（未指定 image）");

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let title = Paragraph::new(Line::from(vec![Span::styled(
                "Rust SPFDisk TUI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);

            let body_text = match screen {
                Screen::Main => format!(
                    "目標磁碟: {disk_label}\n\n\
                     [F] 快速分區精靈\n\
                     [1] Windows  [2] macOS  [3] Linux\n\
                     [Q] 離開"
                ),
                Screen::QuickLayout => "快速分區精靈\n\n\
                     [1] Windows UEFI 標準\n\
                     [2] macOS APFS 目標碟\n\
                     [3] Linux ext4 + /home\n\
                     [Esc] 返回"
                    .to_string(),
                Screen::Preview => format!(
                    "分區草稿預覽\n\n\
                     模板: {selected_template}\n\
                     磁碟: {disk_label}\n\n\
                     （預覽模式 — 未寫入）\n\
                     [Esc] 返回  [Q] 離開"
                ),
            };

            let body = Paragraph::new(body_text)
                .block(Block::default().borders(Borders::ALL).title("主畫面"));
            f.render_widget(body, chunks[1]);

            let footer = Paragraph::new("按 Q 離開 | 預設唯讀，寫入需 W + 確認")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match (screen, key.code) {
                    (Screen::Main, KeyCode::Char('q') | KeyCode::Char('Q')) => break,
                    (Screen::Main, KeyCode::Char('f') | KeyCode::Char('F')) => {
                        screen = Screen::QuickLayout;
                    }
                    (Screen::Main, KeyCode::Char('1')) => {
                        selected_template = "Windows UEFI 標準";
                        screen = Screen::Preview;
                    }
                    (Screen::Main, KeyCode::Char('2')) => {
                        selected_template = "macOS APFS 目標碟";
                        screen = Screen::Preview;
                    }
                    (Screen::Main, KeyCode::Char('3')) => {
                        selected_template = "Linux ext4 + /home";
                        screen = Screen::Preview;
                    }
                    (Screen::QuickLayout, KeyCode::Esc) => screen = Screen::Main,
                    (Screen::QuickLayout, KeyCode::Char('1')) => {
                        selected_template = "Windows UEFI 標準";
                        screen = Screen::Preview;
                    }
                    (Screen::QuickLayout, KeyCode::Char('2')) => {
                        selected_template = "macOS APFS 目標碟";
                        screen = Screen::Preview;
                    }
                    (Screen::QuickLayout, KeyCode::Char('3')) => {
                        selected_template = "Linux ext4 + /home";
                        screen = Screen::Preview;
                    }
                    (Screen::Preview, KeyCode::Esc) => screen = Screen::QuickLayout,
                    (Screen::Preview, KeyCode::Char('q') | KeyCode::Char('Q')) => break,
                    _ => {}
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
