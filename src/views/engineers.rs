use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::engineering_data::{self, EngineerInfo, Modification};
use crate::event_shim::{KeyCode, KeyEvent};
use crate::todo::{ModKind, TodoItem, TodoList};
use crate::views::ViewEvent;

#[derive(Clone, PartialEq)]
enum Panel {
    Engineers,
    Mods,
}

#[derive(Clone, PartialEq)]
enum Kind {
    Ship,
    OnFoot,
}

pub struct EngineersView {
    kind: Kind,
    engineer_idx: usize,
    mod_idx: usize,
    grade: u8,
    panel: Panel,
    pub todo: TodoList,
    status: String,
    scroll_offset: usize,
}

impl EngineersView {
    pub fn new() -> Self {
        Self {
            kind: Kind::Ship,
            engineer_idx: 0,
            mod_idx: 0,
            grade: 5,
            panel: Panel::Engineers,
            todo: TodoList::load(),
            status: String::new(),
            scroll_offset: 0,
        }
    }

    fn engineers(&self) -> &'static [EngineerInfo] {
        match self.kind {
            Kind::Ship => &engineering_data::engineers().ship,
            Kind::OnFoot => &engineering_data::engineers().onfoot,
        }
    }

    fn selected_engineer(&self) -> Option<&'static EngineerInfo> {
        self.engineers().get(self.engineer_idx)
    }

    fn mods_for_selected(&self) -> Vec<&'static Modification> {
        let eng = match self.selected_engineer() {
            Some(e) => e,
            None => return vec![],
        };
        let all_mods = match self.kind {
            Kind::Ship => &engineering_data::modifications().ship,
            Kind::OnFoot => &engineering_data::modifications().onfoot,
        };
        all_mods
            .iter()
            .filter(|m| m.engineer_ids.contains(&eng.id))
            .collect()
    }

    fn selected_mod(&self) -> Option<&'static Modification> {
        let mods = self.mods_for_selected();
        mods.get(self.mod_idx).copied()
    }

    fn build_engineer_lines(&self) -> Vec<Line<'static>> {
        let engineers = self.engineers();
        let mut lines = Vec::new();
        let kind_label = match self.kind {
            Kind::Ship => "Ship Engineers",
            Kind::OnFoot => "On-Foot Engineers",
        };
        lines.push(Line::from(Span::styled(
            format!(" {} (t: toggle) ", kind_label),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        for (i, eng) in engineers.iter().enumerate() {
            let selected = i == self.engineer_idx && self.panel == Panel::Engineers;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!(" {} ", eng.name),
                style,
            )));
        }
        lines
    }

    fn build_mod_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let eng = match self.selected_engineer() {
            Some(e) => e,
            None => {
                lines.push(Line::from("No engineer selected."));
                return lines;
            }
        };
        lines.push(Line::from(Span::styled(
            format!(" {} — {} / {} ", eng.name, eng.system, eng.station),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!(" Unlock: {}", eng.unlock),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let mods = self.mods_for_selected();
        if mods.is_empty() {
            lines.push(Line::from("No modifications listed for this engineer."));
            return lines;
        }

        lines.push(Line::from(Span::styled(
            " Modifications (a/d: grade, 'a': add to todo) ",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let mut current_type: Option<&str> = None;
        for (i, m) in mods.iter().enumerate() {
            if Some(m.module_type.as_str()) != current_type {
                current_type = Some(&m.module_type);
                lines.push(Line::from(Span::styled(
                    format!(" — {} —", m.module_type),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                )));
            }
            let selected = i == self.mod_idx && self.panel == Panel::Mods;
            let grade = self.grade.min(m.max_grade);
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!(" {}  G{}  — {}", m.name, grade, m.effect),
                style,
            )));

            if selected {
                let grade_key = grade.to_string();
                if let Some(costs) = m.grades.get(&grade_key) {
                    lines.push(Line::from(Span::styled(
                        format!("   Materials (G{}):", grade),
                        Style::default().fg(Color::DarkGray),
                    )));
                    for cost in costs {
                        lines.push(Line::from(Span::styled(
                            format!("     • {} x{}", cost.name, cost.count),
                            Style::default().fg(Color::Gray),
                        )));
                    }
                }
            }
        }
        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        self.status.clear();
        match key.code {
            KeyCode::Char('t') => {
                self.kind = match self.kind {
                    Kind::Ship => Kind::OnFoot,
                    Kind::OnFoot => Kind::Ship,
                };
                self.engineer_idx = 0;
                self.mod_idx = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Tab => {
                self.panel = match self.panel {
                    Panel::Engineers => Panel::Mods,
                    Panel::Mods => Panel::Engineers,
                };
                self.scroll_offset = 0;
            }
            KeyCode::Char('w') | KeyCode::Up => match self.panel {
                Panel::Engineers => {
                    if self.engineer_idx > 0 {
                        self.engineer_idx -= 1;
                        self.mod_idx = 0;
                        self.scroll_offset = 0;
                    }
                }
                Panel::Mods => {
                    if self.mod_idx > 0 {
                        self.mod_idx -= 1;
                    } else if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.panel {
                Panel::Engineers => {
                    if self.engineer_idx + 1 < self.engineers().len() {
                        self.engineer_idx += 1;
                        self.mod_idx = 0;
                        self.scroll_offset = 0;
                    }
                }
                Panel::Mods => {
                    let count = self.mods_for_selected().len();
                    if self.mod_idx + 1 < count {
                        self.mod_idx += 1;
                    }
                }
            },
            KeyCode::Char('a') | KeyCode::Left => {
                if self.panel == Panel::Mods {
                    if self.grade > 1 {
                        self.grade -= 1;
                    }
                }
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if self.panel == Panel::Mods {
                    if let Some(m) = self.selected_mod() {
                        if self.grade < m.max_grade {
                            self.grade += 1;
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if self.panel == Panel::Mods {
                    if let Some(m) = self.selected_mod() {
                        let grade = self.grade.min(m.max_grade);
                        let kind = match self.kind {
                            Kind::Ship => ModKind::Ship,
                            Kind::OnFoot => ModKind::OnFoot,
                        };
                        self.todo.add(TodoItem {
                            mod_id: m.id.clone(),
                            grade,
                            mod_name: m.name.clone(),
                            module_type: m.module_type.clone(),
                            kind,
                        });
                        self.status = format!("Added {} G{} to todo list", m.name, grade);
                    }
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(10)])
            .split(area);

        let eng_lines = self.build_engineer_lines();
        let left_block = Block::default()
            .title(" Engineers ")
            .borders(Borders::ALL)
            .style(if self.panel == Panel::Engineers {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            });
        frame.render_widget(
            Paragraph::new(eng_lines).block(left_block),
            chunks[0],
        );

        let mod_lines = self.build_mod_lines();
        let visible = area.height.saturating_sub(4) as usize;
        let max_scroll = mod_lines.len().saturating_sub(visible);
        let offset = self.scroll_offset.min(max_scroll) as u16;
        let status = if self.status.is_empty() {
            " Mods (tab: panel, enter: add to todo) ".to_string()
        } else {
            format!(" {} ", self.status)
        };
        let right_block = Block::default()
            .title(status)
            .borders(Borders::ALL)
            .style(if self.panel == Panel::Mods {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            });
        frame.render_widget(
            Paragraph::new(mod_lines).block(right_block).scroll((offset, 0)),
            chunks[1],
        );
    }
}
