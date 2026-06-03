use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::views::engineers::EngineersView;
use crate::views::inventory::InventoryView;
use crate::views::ViewEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[derive(Clone, Copy, PartialEq)]
enum SubTab { Inventory, Engineers }

pub struct WorkshopView {
    tab: SubTab,
    pub inventory: InventoryView,
    pub engineers: EngineersView,
}

impl WorkshopView {
    pub fn new() -> Self {
        Self {
            tab: SubTab::Inventory,
            inventory: InventoryView::new(),
            engineers: EngineersView::new(),
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('1') => { self.tab = SubTab::Inventory;  return ViewEvent::Consumed; }
            KeyCode::Char('2') => { self.tab = SubTab::Engineers; return ViewEvent::Consumed; }
            _ => {}
        }
        match self.tab {
            SubTab::Inventory => self.inventory.handle_key(key),
            SubTab::Engineers => self.engineers.handle_key(key),
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
            Span::styled(" Inventory ", if self.tab == SubTab::Inventory { active } else { inactive }),
            Span::raw("  "),
            Span::styled(" Engineers ", if self.tab == SubTab::Engineers { active } else { inactive }),
            Span::raw("  "),
            Span::styled("1/2: switch", hint),
        ])), chunks[0]);

        match self.tab {
            SubTab::Inventory => self.inventory.render(frame, chunks[1], journal),
            SubTab::Engineers => self.engineers.render(frame, chunks[1]),
        }
    }
}
