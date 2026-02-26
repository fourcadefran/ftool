pub mod data_inspector;
pub mod file_browser;
pub mod home;
pub mod json_inspector;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tui::app::{App, Popup};

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Renders the tippecanoe PMTiles configuration popup.
/// Call this at the end of any view's `render()` that can trigger the popup.
pub fn render_pmtiles_popup(frame: &mut Frame, app: &App, area: Rect) {
    let Popup::PmtilesConfig { source_file, config, preset, selected_field } = &app.popup else {
        return;
    };

    let popup_area = centered_rect(56, 14, area);
    frame.render_widget(Clear, popup_area);

    let filename = source_file
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(format!(" Convert to PMTiles: {} ", filename))
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let highlight = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let normal = Style::default().fg(Color::White);

    let fields: [(String, bool); 6] = [
        (format!("  Preset            < {} >", preset.label()), false),
        (format!("  Min Zoom          < {:2} >", config.min_zoom), false),
        (format!("  Max Zoom          < {:2} >", config.max_zoom), false),
        (
            format!("  No Feature Limit  [{}]", if config.no_feature_limit { "x" } else { " " }),
            true,
        ),
        (
            format!("  No Tile Size Limit[{}]", if config.no_tile_size_limit { "x" } else { " " }),
            true,
        ),
        (
            format!("  Drop Densest      [{}]", if config.drop_densest_as_needed { "x" } else { " " }),
            true,
        ),
    ];

    let mut lines = vec![Line::from("")];
    for (i, (label, _)) in fields.iter().enumerate() {
        let style = if i == *selected_field { highlight } else { normal };
        lines.push(Line::from(Span::styled(label.clone(), style)));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Enter ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("convert  "),
        Span::styled(" Esc ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("cancel  "),
        Span::styled(" \u{2190}\u{2192} ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("adjust"),
    ]));

    frame.render_widget(Paragraph::new(lines), inner);
}
