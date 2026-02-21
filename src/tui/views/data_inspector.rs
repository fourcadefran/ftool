use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Tabs};

use crate::tui::app::{App, InspectorTab, Popup};
use crate::tui::views::centered_rect;
use crate::tui::widgets::status_bar;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let status_area = chunks[1];

    // Title with file name and row count
    let title = if let Some(ref file) = app.inspector_file {
        let name = file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        format!(" Inspector: {} ({} rows) ", name, app.inspector_row_count)
    } else {
        " Inspector ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(main_area);
    frame.render_widget(block, main_area);

    // Tabs + content
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    // Tab bar
    let tab_index = match app.inspector_tab {
        InspectorTab::Schema => 0,
        InspectorTab::Preview => 1,
    };
    let tabs = Tabs::new(vec!["Schema", "Preview"])
        .select(tab_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");
    frame.render_widget(tabs, inner_chunks[0]);

    // Content area
    match app.inspector_tab {
        InspectorTab::Schema => render_schema(frame, app, inner_chunks[1]),
        InspectorTab::Preview => render_preview(frame, app, inner_chunks[1]),
    }

    // Status bar
    status_bar::render(
        frame,
        status_area,
        &[
            ("Tab", "switch"),
            ("\u{2191}\u{2193}", "scroll"),
            ("c", "convert"),
            ("Esc", "back"),
            ("q", "quit"),
        ],
    );

    // Render popup on top if active
    render_popup(frame, app, frame.area());
}

fn render_schema(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Column Name", "Type", "Nulls", "Min", "Max", "Avg"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .inspector_schema
        .iter()
        .enumerate()
        .skip(app.inspector_scroll)
        .map(|(i, (name, dtype))| {
            let null_count = app
                .inspector_null_counts
                .get(i)
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".to_string());
            let min = app
                .inspector_min_values
                .get(i)
                .cloned()
                .unwrap_or_else(|| "-".to_string());
            let max = app
                .inspector_max_values
                .get(i)
                .cloned()
                .unwrap_or_else(|| "-".to_string());
            let mean = app
                .inspector_mean_values
                .get(i)
                .cloned()
                .unwrap_or_else(|| "-".to_string());
            Row::new(vec![
                name.clone(),
                dtype.clone(),
                null_count,
                min,
                max,
                mean,
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Min(15),
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(header);

    frame.render_widget(table, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    if app.inspector_preview_headers.is_empty() {
        let msg =
            Paragraph::new("No preview data available").style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    // Build header row
    let header = Row::new(app.inspector_preview_headers.clone())
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    // Build data rows with scroll offset
    let rows: Vec<Row> = app
        .inspector_preview_data
        .iter()
        .skip(app.inspector_scroll)
        .map(|row_data| Row::new(row_data.clone()))
        .collect();

    // Column widths - distribute evenly
    let col_count = app.inspector_preview_headers.len();
    let widths: Vec<Constraint> = (0..col_count).map(|_| Constraint::Min(10)).collect();

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, area);
}

fn render_popup(frame: &mut Frame, app: &App, area: Rect) {
    match &app.popup {
        Popup::None => {}
        Popup::ConvertConfirm { target_format } => {
            let popup_area = centered_rect(44, 7, area);
            frame.render_widget(Clear, popup_area);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Convert ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

            let inner = block.inner(popup_area);
            frame.render_widget(block, popup_area);

            let text = vec![
                Line::from(""),
                Line::from(format!("  Convert to {}?", target_format)),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " Enter ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("confirm  "),
                    Span::styled(
                        " Esc ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("cancel"),
                ]),
            ];
            frame.render_widget(Paragraph::new(text), inner);
        }
        Popup::Message { title, body } => {
            let width = (body.len() as u16 + 6)
                .max(30)
                .min(area.width.saturating_sub(4));
            let popup_area = centered_rect(width, 7, area);
            frame.render_widget(Clear, popup_area);

            let color = if title.contains("Error") {
                Color::Red
            } else {
                Color::Green
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .title(format!(" {} ", title))
                .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD));

            let inner = block.inner(popup_area);
            frame.render_widget(block, popup_area);

            let text = vec![
                Line::from(""),
                Line::from(format!("  {}", body)),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        " Enter/Esc ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("close"),
                ]),
            ];
            frame.render_widget(Paragraph::new(text), inner);
        }
    }
}
