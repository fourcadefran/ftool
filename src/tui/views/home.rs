use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

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

    // Outer block
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" ftool ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(main_area);
    frame.render_widget(block, main_area);

    // Vertical layout: title + spacer + menu
    let v_pad = inner.height.saturating_sub(6) / 2;
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Length(2), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Menu items
            Constraint::Min(0),
        ])
        .split(inner);

    // Title
    let title = Paragraph::new("ftool - CLI Toolbox")
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, inner_chunks[1]);

    // Subtitle
    let subtitle = Paragraph::new("Select an action:")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(subtitle, inner_chunks[2]);

    // Menu items
    let items = vec![
        ListItem::new("  Browse Files"),
        ListItem::new("  Inspect Data File"),
    ];

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.home_selected));

    // Center list horizontally
    let list_width = 30u16.min(inner_chunks[3].width);
    let h_pad = inner_chunks[3].width.saturating_sub(list_width) / 2;
    let list_area = ratatui::layout::Rect::new(
        inner_chunks[3].x + h_pad,
        inner_chunks[3].y,
        list_width,
        inner_chunks[3].height,
    );

    frame.render_stateful_widget(list, list_area, &mut state);

    // Status bar
    status_bar::render(
        frame,
        status_area,
        &[("\u{2191}\u{2193}", "navigate"), ("Enter", "select"), ("q", "quit")],
    );
}
