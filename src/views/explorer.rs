use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct ExplorerView;

impl ExplorerView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _key: &KeyEvent) -> ViewEvent {
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "System Explorer",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Explore star systems and celestial bodies."),
            Line::from(""),
            Line::from(Span::styled(
                "Status: Coming Soon",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Planned features:"),
            Line::from("  - System scanning"),
            Line::from("  - Body information"),
            Line::from("  - Scan data analysis"),
            Line::from("  - Valuable body detection"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Explorer ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
