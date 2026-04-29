use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct StationsView;

impl StationsView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _key: &KeyEvent) -> ViewEvent {
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "Stations",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("View station information and services."),
            Line::from(""),
            Line::from(Span::styled(
                "Status: Coming Soon",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Planned features:"),
            Line::from("  - Station services overview"),
            Line::from("  - Market data"),
            Line::from("  - Outfitting information"),
            Line::from("  - Shipyard listings"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Stations ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
