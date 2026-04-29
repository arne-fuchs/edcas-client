use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

pub struct MaterialsView;

impl MaterialsView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, _key: &KeyEvent) -> ViewEvent {
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "Material Inventory",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Track your ship's material inventory."),
            Line::from(""),
            Line::from(Span::styled(
                "Status: Coming Soon",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Planned features:"),
            Line::from("  - Material count tracking"),
            Line::from("  - Engineer requirements"),
            Line::from("  - Material synthesis"),
            Line::from("  - Trade-in assistance"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Materials ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
