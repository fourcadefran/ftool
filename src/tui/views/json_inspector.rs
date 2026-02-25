use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Tabs};

use crate::commands::json_inspector::FileKind;
use crate::tui::app::{App, GeoJsonTab, JsonInspectorTab};
use crate::tui::tree::{NodeKind, ScalarType};
use crate::tui::widgets::status_bar;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let status_area = chunks[1];

    let filename = app
        .json_file
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let is_geojson = app.json_kind == Some(FileKind::GeoJson);

    let title = format!(" {} ", filename);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(main_area);
    frame.render_widget(block, main_area);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    if is_geojson {
        render_geo_tabs(frame, app, inner_chunks[0]);
        match app.geo_tab {
            GeoJsonTab::Summary => render_geo_summary(frame, app, inner_chunks[1]),
            GeoJsonTab::Features => render_features_table(frame, app, inner_chunks[1]),
            GeoJsonTab::Tree => render_tree(frame, app, inner_chunks[1]),
        }
        status_bar::render(frame, status_area, &[
            ("Tab", "next tab"),
            ("\u{2191}\u{2193}", "scroll"),
            ("Enter", "expand/collapse"),
            ("Esc", "back"),
            ("q", "quit"),
        ]);
    } else {
        render_json_tabs(frame, app, inner_chunks[0]);
        match app.json_tab {
            JsonInspectorTab::Tree => render_tree(frame, app, inner_chunks[1]),
            JsonInspectorTab::Raw => render_raw(frame, app, inner_chunks[1]),
        }
        status_bar::render(frame, status_area, &[
            ("Tab", "switch"),
            ("\u{2191}\u{2193}", "scroll"),
            ("Enter", "expand/collapse"),
            ("Esc", "back"),
            ("q", "quit"),
        ]);
    }
}

fn render_json_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let idx = match app.json_tab {
        JsonInspectorTab::Tree => 0,
        JsonInspectorTab::Raw => 1,
    };
    let tabs = Tabs::new(vec!["Tree", "Raw"])
        .select(idx)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider("|");
    frame.render_widget(tabs, area);
}

fn render_geo_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let idx = match app.geo_tab {
        GeoJsonTab::Summary => 0,
        GeoJsonTab::Features => 1,
        GeoJsonTab::Tree => 2,
    };
    let tabs = Tabs::new(vec!["Summary", "Features", "Tree"])
        .select(idx)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .divider("|");
    frame.render_widget(tabs, area);
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let nodes = &app.json_tree_nodes;
    let scroll = app.json_scroll;

    let lines: Vec<Line> = nodes
        .iter()
        .enumerate()
        .skip(scroll)
        .map(|(i, (_, node))| {
            let indent = "  ".repeat(node.depth);
            let key_part = match &node.key {
                Some(k) => format!("{}: ", k),
                None => String::new(),
            };
            let is_selected = i == scroll;
            let bg = if is_selected { Color::DarkGray } else { Color::Reset };

            match &node.kind {
                NodeKind::Object => {
                    let arrow = if node.collapsed { "\u{25b6}" } else { "\u{25bc}" };
                    Line::from(vec![
                        Span::raw(indent).style(Style::default().bg(bg)),
                        Span::styled(arrow, Style::default().fg(Color::Yellow).bg(bg)),
                        Span::styled(format!(" {}", key_part), Style::default().fg(Color::Cyan).bg(bg)),
                        Span::styled(
                            format!("{{{}}}", node.child_count),
                            Style::default().fg(Color::DarkGray).bg(bg),
                        ),
                    ])
                }
                NodeKind::Array => {
                    let arrow = if node.collapsed { "\u{25b6}" } else { "\u{25bc}" };
                    Line::from(vec![
                        Span::raw(indent).style(Style::default().bg(bg)),
                        Span::styled(arrow, Style::default().fg(Color::Yellow).bg(bg)),
                        Span::styled(format!(" {}", key_part), Style::default().fg(Color::Cyan).bg(bg)),
                        Span::styled(
                            format!("[{}]", node.child_count),
                            Style::default().fg(Color::DarkGray).bg(bg),
                        ),
                    ])
                }
                NodeKind::Scalar(val, scalar_type) => {
                    let val_color = match scalar_type {
                        ScalarType::String => Color::Yellow,
                        ScalarType::Number => Color::Cyan,
                        ScalarType::Bool => Color::Green,
                        ScalarType::Null => Color::DarkGray,
                    };
                    Line::from(vec![
                        Span::raw(format!("{}  ", indent)).style(Style::default().bg(bg)),
                        Span::styled(key_part, Style::default().fg(Color::White).bg(bg)),
                        Span::styled(val.clone(), Style::default().fg(val_color).bg(bg)),
                    ])
                }
            }
        })
        .collect();

    let para = Paragraph::new(lines);
    frame.render_widget(para, area);
}

fn render_raw(frame: &mut Frame, app: &App, area: Rect) {
    let lines: Vec<Line> = app
        .json_raw
        .lines()
        .skip(app.json_scroll)
        .map(|l| Line::from(l.to_string()))
        .collect();
    let para = Paragraph::new(lines).style(Style::default().fg(Color::Gray));
    frame.render_widget(para, area);
}

fn render_geo_summary(frame: &mut Frame, app: &App, area: Rect) {
    let text = match &app.json_geosummary {
        None => vec![Line::from("No GeoJSON summary available")],
        Some((count, types, bbox)) => {
            let mut lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Features:  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(count.to_string()),
                ]),
                Line::from(vec![
                    Span::styled("  Geometry:  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(types.join(", ")),
                ]),
            ];
            if let Some((min_lon, min_lat, max_lon, max_lat)) = bbox {
                lines.push(Line::from(""));
                lines.push(Line::from(
                    Span::styled("  Bounding Box:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                ));
                lines.push(Line::from(format!("    Min lon/lat: {:.6}, {:.6}", min_lon, min_lat)));
                lines.push(Line::from(format!("    Max lon/lat: {:.6}, {:.6}", max_lon, max_lat)));
            }
            lines
        }
    };
    let para = Paragraph::new(text);
    frame.render_widget(para, area);
}

fn render_features_table(frame: &mut Frame, app: &App, area: Rect) {
    if app.json_features_headers.is_empty() {
        let msg = Paragraph::new("No features or no properties").style(Style::default().fg(Color::Gray));
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(app.json_features_headers.clone())
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .json_features_data
        .iter()
        .skip(app.json_scroll)
        .map(|row| Row::new(row.clone()))
        .collect();

    let col_count = app.json_features_headers.len();
    let widths: Vec<Constraint> = (0..col_count).map(|_| Constraint::Min(12)).collect();

    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, area);
}
