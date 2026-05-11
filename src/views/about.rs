use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::views::ViewEvent;

pub struct AboutView;

impl AboutView {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Tab => ViewEvent::NextTab,
            KeyCode::BackTab => ViewEvent::PrevTab,
            _ => ViewEvent::None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(Span::styled(
                "EDCAS - Elite Dangerous Commander Assistant System",
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Version 0.4.0",
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "A terminal-based assistant for Elite Dangerous commanders.",
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Features:",
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  - Galnet news feed"),
            Line::from("  - System explorer"),
            Line::from("  - Mining assistance"),
            Line::from("  - Material inventory tracking"),
            Line::from("  - Station information"),
            Line::from("  - Carrier management"),
            Line::from("  - Journal log processing"),
            Line::from("  - EDDN integration"),
            Line::from(""),
            Line::from(Span::styled(
                "Repository: https://github.com/arne-fuchs/edcas-client",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Controls:",
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  w/s    - Navigate up/down"),
            Line::from("  a      - Focus sidebar"),
            Line::from("  d      - Focus content/fields"),
            Line::from("  j/k    - Navigate columns in icon sections"),
            Line::from("  space  - Select section or edit field"),
            Line::from("  enter  - Confirm edit"),
            Line::from("  esc    - Cancel edit"),
            Line::from("  tab    - Next tab"),
            Line::from("  shift+tab - Previous tab"),
            Line::from("  x      - Quit"),
        ];

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" About ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}
