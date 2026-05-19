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
    /// Flat index into all (mod, grade) pairs for the selected engineer.
    mod_grade_idx: usize,
    panel: Panel,
    pub todo: TodoList,
    status: String,
}

impl EngineersView {
    pub fn new() -> Self {
        Self {
            kind: Kind::Ship,
            engineer_idx: 0,
            mod_grade_idx: 0,
            panel: Panel::Engineers,
            todo: TodoList::load(),
            status: String::new(),
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

    fn flat_mod_grades(&self) -> Vec<(&'static Modification, u8)> {
        self.mods_for_selected()
            .into_iter()
            .flat_map(|m| {
                // On-foot mods are always applied at max grade — no intermediate grades.
                let grades: Box<dyn Iterator<Item = u8>> = match self.kind {
                    Kind::OnFoot => Box::new(std::iter::once(m.max_grade)),
                    Kind::Ship => Box::new(1..=m.max_grade),
                };
                grades.map(move |g| (m, g))
            })
            .collect()
    }

    fn build_engineer_lines(&self) -> Vec<Line<'static>> {
        let engineers = self.engineers();
        let mut lines = Vec::new();
        let kind_label = match self.kind {
            Kind::Ship => "Ship Engineers",
            Kind::OnFoot => "On-Foot Engineers",
        };
        lines.push(Line::from(Span::styled(
            format!(" {} (t: toggle, d: mods) ", kind_label),
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

    /// Returns (lines, selected_line_index) so render can auto-scroll to the selection.
    fn build_mod_lines(&self) -> (Vec<Line<'static>>, usize) {
        let mut lines = Vec::new();
        let mut selected_line = 0usize;

        let eng = match self.selected_engineer() {
            Some(e) => e,
            None => {
                lines.push(Line::from("No engineer selected."));
                return (lines, 0);
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

        let flat = self.flat_mod_grades();
        if flat.is_empty() {
            lines.push(Line::from("No modifications listed for this engineer."));
            return (lines, 0);
        }

        lines.push(Line::from(Span::styled(
            " Modifications (enter: add to todo) ",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let mut current_type: Option<String> = None;
        let mut current_mod_id: Option<String> = None;

        for (i, (m, grade)) in flat.iter().enumerate() {
            // Module-type section header
            if Some(&m.module_type) != current_type.as_ref() {
                if current_type.is_some() {
                    lines.push(Line::from(""));
                }
                current_type = Some(m.module_type.clone());
                lines.push(Line::from(Span::styled(
                    format!(" — {} —", m.module_type),
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                )));
            }

            // Mod name / effect sub-header (once per modification)
            if Some(&m.id) != current_mod_id.as_ref() {
                current_mod_id = Some(m.id.clone());
                lines.push(Line::from(Span::styled(
                    format!("  {}  — {}", m.name, m.effect),
                    Style::default().fg(Color::Yellow),
                )));
            }

            // Grade row — this is the selectable line
            let selected = i == self.mod_grade_idx && self.panel == Panel::Mods;
            if selected {
                selected_line = lines.len();
            }
            let grade_style = if selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let grade_label = match self.kind {
                Kind::OnFoot => "    Materials:".to_string(),
                Kind::Ship => format!("    G{}", grade),
            };
            lines.push(Line::from(Span::styled(grade_label, grade_style)));

            // Materials for this grade
            let grade_key = grade.to_string();
            if let Some(costs) = m.grades.get(&grade_key) {
                for cost in costs {
                    lines.push(Line::from(Span::styled(
                        format!("       • {} x{}", cost.name, cost.count),
                        Style::default().fg(Color::Gray),
                    )));
                }
            }
        }

        (lines, selected_line)
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
                self.mod_grade_idx = 0;
            }
            KeyCode::Tab => {
                self.panel = match self.panel {
                    Panel::Engineers => Panel::Mods,
                    Panel::Mods => Panel::Engineers,
                };
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if self.panel == Panel::Engineers {
                    self.panel = Panel::Mods;
                }
            }
            KeyCode::Char('a') | KeyCode::Left => {
                if self.panel == Panel::Mods {
                    self.panel = Panel::Engineers;
                }
            }
            KeyCode::Char('w') | KeyCode::Up => match self.panel {
                Panel::Engineers => {
                    if self.engineer_idx > 0 {
                        self.engineer_idx -= 1;
                        self.mod_grade_idx = 0;
                    }
                }
                Panel::Mods => {
                    if self.mod_grade_idx > 0 {
                        self.mod_grade_idx -= 1;
                    }
                }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.panel {
                Panel::Engineers => {
                    if self.engineer_idx + 1 < self.engineers().len() {
                        self.engineer_idx += 1;
                        self.mod_grade_idx = 0;
                    }
                }
                Panel::Mods => {
                    let total = self.flat_mod_grades().len();
                    if self.mod_grade_idx + 1 < total {
                        self.mod_grade_idx += 1;
                    }
                }
            },
            KeyCode::PageUp => match self.panel {
                Panel::Engineers => {
                    self.engineer_idx = self.engineer_idx.saturating_sub(10);
                    self.mod_grade_idx = 0;
                }
                Panel::Mods => {
                    self.mod_grade_idx = self.mod_grade_idx.saturating_sub(10);
                }
            },
            KeyCode::PageDown => match self.panel {
                Panel::Engineers => {
                    let max = self.engineers().len().saturating_sub(1);
                    self.engineer_idx = (self.engineer_idx + 10).min(max);
                    self.mod_grade_idx = 0;
                }
                Panel::Mods => {
                    let total = self.flat_mod_grades().len();
                    self.mod_grade_idx = (self.mod_grade_idx + 10).min(total.saturating_sub(1));
                }
            },
            KeyCode::Enter => {
                if self.panel == Panel::Mods {
                    let flat = self.flat_mod_grades();
                    if let Some((m, grade)) = flat.get(self.mod_grade_idx) {
                        let kind = match self.kind {
                            Kind::Ship => ModKind::Ship,
                            Kind::OnFoot => ModKind::OnFoot,
                        };
                        self.todo.add(TodoItem {
                            mod_id: m.id.clone(),
                            grade: *grade,
                            mod_name: m.name.clone(),
                            module_type: m.module_type.clone(),
                            kind,
                            count: 1,
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

        let (mod_lines, selected_line) = self.build_mod_lines();
        let visible = area.height.saturating_sub(4) as usize;
        let offset = selected_line.saturating_sub(visible / 3).min(
            mod_lines.len().saturating_sub(visible)
        ) as u16;

        let status = if self.status.is_empty() {
            " Mods (a: engineers, w/s: navigate, enter: add to todo) ".to_string()
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
