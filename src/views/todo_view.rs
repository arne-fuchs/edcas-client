use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::engineering_data::{self, MaterialCost};
use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::todo::{ModKind, TodoList};
use crate::views::ViewEvent;

pub struct TodoView {
    pub todo: TodoList,
    selected_idx: usize,
    scroll_offset: usize,
}

impl TodoView {
    pub fn new() -> Self {
        Self {
            todo: TodoList::load(),
            selected_idx: 0,
            scroll_offset: 0,
        }
    }

    pub fn reload(&mut self) {
        self.todo = TodoList::load();
    }

    fn grade_costs(mod_id: &str, grade: u8, kind: &ModKind) -> Vec<MaterialCost> {
        let all_mods = match kind {
            ModKind::Ship => &engineering_data::modifications().ship,
            ModKind::OnFoot => &engineering_data::modifications().onfoot,
        };
        all_mods
            .iter()
            .find(|m| m.id == mod_id)
            .and_then(|m| m.grades.get(&grade.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    fn have_count(mat_name: &str, kind: &ModKind, journal: &JournalData) -> i32 {
        match kind {
            ModKind::Ship => {
                let lower = mat_name.to_lowercase();
                journal
                    .materials_raw
                    .iter()
                    .chain(journal.materials_manufactured.iter())
                    .chain(journal.materials_encoded.iter())
                    .find(|i| i.name.to_lowercase() == lower)
                    .map(|i| i.count)
                    .unwrap_or(0)
            }
            ModKind::OnFoot => {
                let lower = mat_name.to_lowercase();
                journal
                    .shiplocker
                    .items
                    .iter()
                    .chain(journal.shiplocker.components.iter())
                    .chain(journal.shiplocker.consumables.iter())
                    .chain(journal.shiplocker.data.iter())
                    .chain(journal.backpack.items.iter())
                    .chain(journal.backpack.components.iter())
                    .chain(journal.backpack.consumables.iter())
                    .chain(journal.backpack.data.iter())
                    .find(|i| i.localised.to_lowercase() == lower || i.name.to_lowercase() == lower)
                    .map(|i| i.count)
                    .unwrap_or(0)
            }
        }
    }

    fn build_left_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            " Todo List ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if self.todo.items.is_empty() {
            lines.push(Line::from(Span::styled(
                " No items yet. Add mods in the Engineers tab.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        for (i, item) in self.todo.items.iter().enumerate() {
            let selected = i == self.selected_idx;
            let kind_tag = match item.kind {
                ModKind::Ship => "ship",
                ModKind::OnFoot => "foot",
            };
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!(" [{}] {} G{}  ({})", i + 1, item.mod_name, item.grade, kind_tag),
                style,
            )));
        }
        lines
    }

    fn build_right_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let Some(item) = self.todo.items.get(self.selected_idx) else {
            lines.push(Line::from("Select an item to see material requirements."));
            return lines;
        };

        let kind_label = match item.kind {
            ModKind::Ship => "Ship",
            ModKind::OnFoot => "On-Foot",
        };
        lines.push(Line::from(Span::styled(
            format!(" {} G{}  [{}]", item.mod_name, item.grade, kind_label),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("  Module: {}", item.module_type),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Materials Required:",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));

        let costs = Self::grade_costs(&item.mod_id, item.grade, &item.kind);
        if costs.is_empty() {
            lines.push(Line::from("  No material data found."));
            return lines;
        }

        let mut all_ok = true;
        for cost in &costs {
            let have = Self::have_count(&cost.name, &item.kind, journal);
            let needed = cost.count as i32;
            let ok = have >= needed;
            if !ok {
                all_ok = false;
            }
            let (bar_color, checkmark) = if ok {
                (Color::Green, "✓")
            } else {
                (Color::Red, "✗")
            };
            let bar = progress_bar(have, needed);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} {:<32}", checkmark, cost.name),
                    Style::default().fg(bar_color),
                ),
                Span::styled(
                    format!(" {:>3}/{:<3}", have.min(needed), needed),
                    Style::default().fg(Color::White),
                ),
                Span::styled(format!(" {}", bar), Style::default().fg(bar_color)),
            ]));
        }

        lines.push(Line::from(""));
        if all_ok {
            lines.push(Line::from(Span::styled(
                "  ✓ Ready to engineer!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
        } else {
            let missing = costs
                .iter()
                .filter(|c| Self::have_count(&c.name, &item.kind, journal) < c.count as i32)
                .count();
            lines.push(Line::from(Span::styled(
                format!("  {} material(s) still needed.", missing),
                Style::default().fg(Color::Yellow),
            )));
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    self.scroll_offset = 0;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_idx + 1 < self.todo.items.len() {
                    self.selected_idx += 1;
                    self.scroll_offset = 0;
                }
            }
            KeyCode::Delete | KeyCode::Char('x') => {
                if self.selected_idx < self.todo.items.len() {
                    self.todo.remove(self.selected_idx);
                    if self.selected_idx > 0 && self.selected_idx >= self.todo.items.len() {
                        self.selected_idx -= 1;
                    }
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(36), Constraint::Min(10)])
            .split(area);

        let left_lines = self.build_left_lines();
        frame.render_widget(
            Paragraph::new(left_lines).block(
                Block::default()
                    .title(" Todo (x: remove) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Cyan)),
            ),
            chunks[0],
        );

        let right_lines = self.build_right_lines(journal);
        let visible = area.height.saturating_sub(2) as usize;
        let max_scroll = right_lines.len().saturating_sub(visible);
        let offset = self.scroll_offset.min(max_scroll) as u16;
        frame.render_widget(
            Paragraph::new(right_lines)
                .block(
                    Block::default()
                        .title(" Materials (w/s: navigate) ")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White)),
                )
                .scroll((offset, 0)),
            chunks[1],
        );
    }
}

fn progress_bar(have: i32, need: i32) -> String {
    let ratio = if need <= 0 { 1.0 } else { (have as f64 / need as f64).min(1.0) };
    let filled = (ratio * 12.0).round() as usize;
    let empty = 12usize.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
