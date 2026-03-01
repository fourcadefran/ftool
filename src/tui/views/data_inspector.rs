use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Tabs};

use crate::tui::app::{App, FilterEditorState, FilterField, InspectorTab, Popup, FILTER_OPERATORS};
use crate::tui::views::centered_rect;
use crate::tui::widgets::status_bar;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let info_area = chunks[1];
    let status_area = chunks[2];

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

    // Info bar (only in Preview tab)
    if app.inspector_tab == InspectorTab::Preview {
        const PAGE_SIZE: usize = 50;
        let from = app.inspector_page * PAGE_SIZE + 1;
        let to = ((app.inspector_page + 1) * PAGE_SIZE).min(app.inspector_row_count);
        let total_pages = (app.inspector_row_count + PAGE_SIZE - 1) / PAGE_SIZE;

        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)])
            .split(info_area);

        let left = Paragraph::new(format!(" showing {} to {} of {} ", from, to, app.inspector_row_count))
            .style(Style::default().fg(Color::DarkGray));
        let right = Paragraph::new(format!(" page {} of {} ", app.inspector_page + 1, total_pages))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right);

        frame.render_widget(left, info_chunks[0]);
        frame.render_widget(right, info_chunks[2]);

        if !app.inspector_filters.is_empty() {
            let n = app.inspector_filters.len();
            let label = if n == 1 { "filter".to_string() } else { format!("{} filters", n) };
            let center = Paragraph::new(format!(" {} active ", label))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            frame.render_widget(center, info_chunks[1]);
        }
    }

    // Status bar
    let mut hints: Vec<(&str, &str)> = vec![
        ("Tab", "Switch"),
        ("\u{2191}\u{2193}", "Scroll"),
    ];
    if app.inspector_tab == InspectorTab::Preview {
        hints.push(("\u{2190}", "Previous page"));
        hints.push(("\u{2192}", "Next page"));
        hints.push(("f", "filter"));
    }
    hints.extend_from_slice(&[
        ("c", "Convert"),
        ("Esc", "Back"),
        ("q", "Quit")
    ]);

    status_bar::render(frame, status_area, &hints);

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
        Popup::FilterEditor(state) => render_filter_popup(frame, app, state, area),
    }
}

fn render_filter_popup(frame: &mut Frame, app: &App, state: &FilterEditorState, area: Rect) {
    let width = 72_u16.min(area.width.saturating_sub(4));
    let height = 16_u16.min(area.height.saturating_sub(2));
    let popup_area = centered_rect(width, height, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Filters ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // conditions list
            Constraint::Length(1),   // separator
            Constraint::Length(3),   // editor fields
            Constraint::Length(1),   // help text
        ])
        .split(inner);

    // --- Conditions list ---
    let mut condition_lines: Vec<Line> = if state.conditions.is_empty() {
        vec![Line::from(Span::styled(
            "  (no filters — Tab to select fields, Enter to add)",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        state.conditions.iter().enumerate().map(|(i, c)| {
            let text = if c.operator == "IS NULL" || c.operator == "IS NOT NULL" {
                format!("  {}. \"{}\" {}", i + 1, c.column, c.operator)
            } else {
                format!("  {}. \"{}\" {} '{}'", i + 1, c.column, c.operator, c.value)
            };
            Line::from(Span::styled(text, Style::default().fg(Color::White)))
        }).collect()
    };
    condition_lines.insert(0, Line::from(Span::styled(
        " Active filters:",
        Style::default().fg(Color::Gray),
    )));
    frame.render_widget(Paragraph::new(condition_lines), chunks[0]);

    // --- Separator ---
    frame.render_widget(
        Paragraph::new(Span::styled(
            "─".repeat(inner.width as usize),
            Style::default().fg(Color::DarkGray),
        )),
        chunks[1],
    );

    // --- Editor fields ---
    let active_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::Gray);

    let col_name = app.inspector_schema
        .get(state.column_idx)
        .map(|(name, _)| name.as_str())
        .unwrap_or("-");
    let op_name = FILTER_OPERATORS.get(state.operator_idx).copied().unwrap_or("=");
    let value_display = format!("{}_", state.value_input);

    let field_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1); 3])
        .split(chunks[2]);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  Column:   "),
            Span::styled(
                format!("[ {:<20} ]", col_name),
                if state.active_field == FilterField::Column { active_style } else { inactive_style },
            ),
            Span::styled("  ↑↓ to change", Style::default().fg(Color::DarkGray)),
        ])),
        field_chunks[0],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  Operator: "),
            Span::styled(
                format!("[ {:<20} ]", op_name),
                if state.active_field == FilterField::Operator { active_style } else { inactive_style },
            ),
            Span::styled("  ↑↓ to change", Style::default().fg(Color::DarkGray)),
        ])),
        field_chunks[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  Value:    "),
            Span::styled(
                format!("[ {:<20} ]", value_display),
                if state.active_field == FilterField::Value { active_style } else { inactive_style },
            ),
            Span::styled("  type to input", Style::default().fg(Color::DarkGray)),
        ])),
        field_chunks[2],
    );

    // --- Help text ---
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(":next  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(":add/apply  "),
            Span::styled("d", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(":remove last  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(":cancel"),
        ])),
        chunks[3],
    );
}
