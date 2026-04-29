use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct CarriersView;

impl CarriersView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _key: &KeyEvent) -> ViewEvent {
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "Fleet Carriers",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Manage your fleet carrier operations."),
            Line::from(""),
            Line::from(Span::styled(
                "Status: Coming Soon",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Planned features:"),
            Line::from("  - Carrier location tracking"),
            Line::from("  - Carrier services overview"),
            Line::from("  - Market data"),
            Line::from("  - Fuel management"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Carriers ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
