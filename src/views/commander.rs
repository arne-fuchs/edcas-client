use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::views::modules::ModulesView;
use crate::views::pilot::PilotView;
use crate::views::ViewEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Clone, Copy, PartialEq)]
enum SubTab { Pilot, Ship }

pub struct CommanderView {
    tab: SubTab,
    pilot: PilotView,
    modules: ModulesView,
}

impl CommanderView {
    pub fn new() -> Self {
        Self {
            tab: SubTab::Pilot,
            pilot: PilotView::new(),
            modules: ModulesView::new(),
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        if key.code == KeyCode::Tab {
            self.tab = match self.tab {
                SubTab::Pilot => SubTab::Ship,
                SubTab::Ship => SubTab::Pilot,
            };
            return ViewEvent::Consumed;
        }
        match self.tab {
            SubTab::Pilot => self.pilot.handle_key(key),
            SubTab::Ship => self.modules.handle_key(key, journal),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let active  = Style::default().fg(Color::Black).bg(crate::theme::accent()).add_modifier(Modifier::BOLD);
        let inactive = Style::default().fg(crate::theme::accent());
        let hint    = Style::default().fg(Color::DarkGray);

        frame.render_widget(Paragraph::new(Line::from(vec![
            Span::styled(" Pilot ", if self.tab == SubTab::Pilot { active } else { inactive }),
            Span::raw("  "),
            Span::styled(" Ship ", if self.tab == SubTab::Ship { active } else { inactive }),
            Span::raw("  "),
            Span::styled("Tab: switch", hint),
        ])), chunks[0]);

        match self.tab {
            SubTab::Pilot => self.pilot.render(frame, chunks[1], journal),
            SubTab::Ship  => self.modules.render(frame, chunks[1], journal),
        }
    }
}
