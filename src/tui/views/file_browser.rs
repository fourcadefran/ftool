use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};

use crate::tui::app::App;
use crate::tui::widgets::status_bar;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Outer block with directory path as title
    let title = format!(" File Browser: {} ", app.current_dir.display());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(main_area);
    frame.render_widget(block, main_area);

    // Two panels: file list (70%) + preview (30%)
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(inner);

    render_file_list(frame, app, panels[0]);
    render_preview(frame, app, panels[1]);

    // Status bar
    status_bar::render(
        frame,
        status_area,
        &[
            ("\u{2191}\u{2193}", "navigate"),
            ("Enter", "open"),
            ("Esc", "back"),
            ("q", "quit"),
        ],
    );
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Size", "Modified"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .dir_entries
        .iter()
        .map(|entry| {
            let name = if entry.is_dir && entry.name != ".." {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };

            let size = if entry.is_dir {
                "<DIR>".to_string()
            } else {
                format_size(entry.size)
            };

            let modified = format_modified(entry.modified);

            let style = if entry.is_dir {
                Style::default().fg(Color::Blue)
            } else {
                match entry
                    .path
                    .extension()
                    .and_then(|e| e.to_str())
                {
                    Some("csv") | Some("parquet") => Style::default().fg(Color::Green),
                    _ => Style::default(),
                }
            };

            Row::new(vec![name, size, modified]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .row_highlight_style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("> ");

    let mut state = TableState::default();
    state.select(Some(app.browser_selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Preview ")
        .title_style(Style::default().fg(Color::Gray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = if let Some(entry) = app.dir_entries.get(app.browser_selected) {
        if entry.name == ".." {
            vec![
                Line::from(Span::styled(
                    "Parent directory",
                    Style::default().fg(Color::Gray),
                )),
            ]
        } else if entry.is_dir {
            vec![
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Gray)),
                    Span::raw("Directory"),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    entry.path.display().to_string(),
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        } else {
            let ext = entry
                .path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Gray)),
                    Span::raw(&entry.name),
                ]),
                Line::from(vec![
                    Span::styled("Size: ", Style::default().fg(Color::Gray)),
                    Span::raw(format_size(entry.size)),
                ]),
                Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::Gray)),
                    Span::raw(ext.to_uppercase()),
                ]),
            ];

            if ext == "csv" || ext == "parquet" {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Press Enter to inspect",
                    Style::default().fg(Color::Green),
                )));
            }

            lines
        }
    } else {
        vec![Line::from(Span::styled(
            "No file selected",
            Style::default().fg(Color::Gray),
        ))]
    };

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn format_modified(time: Option<std::time::SystemTime>) -> String {
    match time {
        Some(t) => {
            let elapsed = t.elapsed().unwrap_or_default();
            let secs = elapsed.as_secs();
            if secs < 60 {
                format!("{}s ago", secs)
            } else if secs < 3600 {
                format!("{}m ago", secs / 60)
            } else if secs < 86400 {
                format!("{}h ago", secs / 3600)
            } else {
                format!("{}d ago", secs / 86400)
            }
        }
        None => "-".to_string(),
    }
}
