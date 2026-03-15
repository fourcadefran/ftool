use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};

use crate::tui::app::App;
use crate::tui::widgets::status_bar;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let constraints = if app.browser_search_active {
        vec![
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        vec![Constraint::Min(0), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let main_area = chunks[0];
    let (search_area, status_area) = if app.browser_search_active {
        (Some(chunks[1]), chunks[2])
    } else {
        (None, chunks[1])
    };

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

    // Search bar
    if let Some(search_area) = search_area {
        render_search_bar(frame, app, search_area);
    }

    // Status bar
    let hints: Vec<(&str, &str)> = if app.browser_search_active {
        vec![
            ("\u{2191}\u{2193}", "navigate"),
            ("Enter", "open"),
            ("Esc", "clear search"),
        ]
    } else {
        vec![
            ("\u{2191}\u{2193}", "navigate"),
            ("Enter", "open"),
            ("Esc", "back"),
            ("/", "search"),
            ("q", "quit"),
        ]
    };
    status_bar::render(frame, status_area, &hints);
}

fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let line = Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Cyan)),
        Span::raw(&app.browser_search_query),
        Span::styled("\u{2588}", Style::default().fg(Color::Gray)),
    ]);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn render_file_list(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Name", "Size", "Modified"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let entries: Vec<&crate::tui::app::DirEntryInfo> = if app.browser_search_active {
        app.browser_filtered_indices
            .iter()
            .filter_map(|&i| app.dir_entries.get(i))
            .collect()
    } else {
        app.dir_entries.iter().collect()
    };

    let rows: Vec<Row> = entries
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
                    Some("csv") | Some("parquet") | Some("json") | Some("geojson") => Style::default().fg(Color::Green),
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

    let actual_index = if app.browser_search_active {
        app.browser_filtered_indices
            .get(app.browser_selected)
            .copied()
    } else {
        Some(app.browser_selected)
    };

    let content = if let Some(entry) = actual_index.and_then(|i| app.dir_entries.get(i)) {
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

            if ext == "csv" || ext == "parquet" || ext == "json" || ext == "geojson" {
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
