use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct MiningView;

impl MiningView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _key: &KeyEvent) -> ViewEvent {
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "Mining Assistant",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Assistance for mining operations."),
            Line::from(""),
            Line::from(Span::styled(
                "Status: Coming Soon",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Planned features:"),
            Line::from("  - Asteroid ring analysis"),
            Line::from("  - Core mining locations"),
            Line::from("  - Profitable material detection"),
            Line::from("  - Hotspot tracking"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Mining ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
