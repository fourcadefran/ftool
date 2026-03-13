use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::tui::app::App;
use crate::tui::widgets::status_bar;

const LOGO: &[&str] = &[
    "███████╗████████╗ ██████╗  ██████╗ ██╗     ",
    "██╔════╝╚══██╔══╝██╔═══██╗██╔═══██╗██║     ",
    "█████╗     ██║   ██║   ██║██║   ██║██║     ",
    "██╔══╝     ██║   ██║   ██║██║   ██║██║     ",
    "██║        ██║   ╚██████╔╝╚██████╔╝███████╗",
    "╚═╝        ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝",
];


const LOGO_WIDTH: u16 = 43;
const OWL_WIDTH: u16 = 9;
const GAP: u16 = 4;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let status_area = chunks[1];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" ftool ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    let inner = block.inner(main_area);
    frame.render_widget(block, main_area);

    // content: 6 (logo+owl) + 1 spacer + 1 subtitle + 1 spacer + 2 menu = 11
    let content_height = 11u16;
    let v_pad = inner.height.saturating_sub(content_height) / 2;

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Length(6), // logo + owl
            Constraint::Length(1), // spacer
            Constraint::Length(1), // subtitle
            Constraint::Length(1), // spacer
            Constraint::Length(2), // menu
            Constraint::Min(0),
        ])
        .split(inner);

    // Center the logo+owl block horizontally
    let total_width = LOGO_WIDTH + GAP + OWL_WIDTH;
    let h_pad = inner.width.saturating_sub(total_width) / 2;

    let logo_owl_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(h_pad),
            Constraint::Length(LOGO_WIDTH),
            Constraint::Length(GAP),
            Constraint::Length(OWL_WIDTH),
            Constraint::Min(0),
        ])
        .split(inner_chunks[1]);

    // Logo
    let logo_lines: Vec<Line> = LOGO
        .iter()
        .map(|&l| Line::from(Span::styled(l, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))))
        .collect();
    frame.render_widget(Paragraph::new(logo_lines), logo_owl_chunks[1]);

    // Owl
    let owl_lines: Vec<Line> = OWL
        .iter()
        .map(|(left, mid, right)| {
            if mid.is_empty() {
                Line::from(Span::styled(*left, Style::default().fg(Color::Yellow)))
            } else {
                Line::from(vec![
                    Span::styled(*left, Style::default().fg(Color::Yellow)),
                    Span::styled(*mid, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(*right, Style::default().fg(Color::Yellow)),
                ])
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(owl_lines), logo_owl_chunks[3]);

    // Subtitle
    frame.render_widget(
        Paragraph::new("Select an action:")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center),
        inner_chunks[3],
    );

    // Menu
    let items = vec![
        ListItem::new("  Browse Files"),
        ListItem::new("  Inspect Data File"),
    ];
    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    let mut state = ListState::default();
    state.select(Some(app.home_selected));

    let list_width = 30u16.min(inner_chunks[5].width);
    let list_h_pad = inner_chunks[5].width.saturating_sub(list_width) / 2;
    let list_area = Rect::new(
        inner_chunks[5].x + list_h_pad,
        inner_chunks[5].y,
        list_width,
        inner_chunks[5].height,
    );
    frame.render_stateful_widget(list, list_area, &mut state);

    // Status bar
    status_bar::render(
        frame,
        status_area,
        &[("\u{2191}\u{2193}", "navigate"), ("Enter", "select"), ("q", "quit")],
    );
}
