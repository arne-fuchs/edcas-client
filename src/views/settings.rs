use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;
use std::str::FromStr;

use crate::desktop::settings::Settings;
use crate::desktop::settings::icons::Icon;
use crate::desktop::settings::journal_reader::ActionAtShutdownSignal;
use crate::views::ViewEvent;

#[derive(Default, Clone, Copy, PartialEq)]
enum SettingsSection {
    #[default]
    JournalReader,
    GraphicsEditor,
    Appearance,
    Explorer,
    Icons,
    Stars,
    Planets,
}

impl SettingsSection {
    fn all() -> Vec<Self> {
        vec![
            SettingsSection::JournalReader,
            SettingsSection::GraphicsEditor,
            SettingsSection::Appearance,
            SettingsSection::Explorer,
            SettingsSection::Icons,
            SettingsSection::Stars,
            SettingsSection::Planets,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            SettingsSection::JournalReader => "Journal Reader",
            SettingsSection::GraphicsEditor => "Graphics Editor",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Explorer => "Explorer",
            SettingsSection::Icons => "Icons",
            SettingsSection::Stars => "Stars",
            SettingsSection::Planets => "Planets",
        }
    }
}

#[derive(Clone, PartialEq)]
enum EditableField {
    StringValue(String),
    BoolValue(bool),
    EnumValue(Vec<&'static str>),
}

struct FieldInfo {
    label: String,
    field: EditableField,
}

struct IconFieldInfo {
    name: String,
    sub_field: usize,
    label: String,
}

pub struct SettingsView {
    section: SettingsSection,
    focus: usize,
    editing: bool,
    edit_buffer: String,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            section: SettingsSection::default(),
            focus: 0,
            editing: false,
            edit_buffer: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, settings: &mut Settings) -> ViewEvent {
        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    self.apply_edit(settings);
                    self.editing = false;
                    self.edit_buffer.clear();
                    return ViewEvent::SettingsChanged;
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.edit_buffer.clear();
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.edit_buffer.pop();
                }
                _ => {}
            }
            return ViewEvent::None;
        }

        let field_count = self.get_field_count(settings);

        match key.code {
            KeyCode::Char('a') => {
                let sections = SettingsSection::all();
                let idx = sections.iter().position(|s| *s == self.section).unwrap_or(0);
                if idx > 0 {
                    self.section = sections[idx - 1];
                    self.focus = 0;
                }
            }
            KeyCode::Char('d') => {
                let sections = SettingsSection::all();
                let idx = sections.iter().position(|s| *s == self.section).unwrap_or(0);
                if idx < sections.len() - 1 {
                    self.section = sections[idx + 1];
                    self.focus = 0;
                }
            }
            KeyCode::Char('w') => {
                if field_count > 0 && self.focus > 0 {
                    self.focus -= 1;
                }
            }
            KeyCode::Char('s') => {
                if field_count > 0 && self.focus < field_count - 1 {
                    self.focus += 1;
                }
            }
            KeyCode::Char(' ') => {
                if field_count > 0 {
                    if self.is_icon_section() {
                        self.edit_icon_field(settings);
                    } else {
                        let fields = self.get_fields(settings);
                        if self.focus < fields.len() {
                            let field = &fields[self.focus].field;
                            match field {
                                EditableField::StringValue(s) => {
                                    self.editing = true;
                                    self.edit_buffer = s.clone();
                                }
                                EditableField::BoolValue(_) => {
                                    self.toggle_bool(self.focus, settings);
                                    return ViewEvent::SettingsChanged;
                                }
                                EditableField::EnumValue(options) => {
                                    self.cycle_enum(self.focus, options, settings);
                                    return ViewEvent::SettingsChanged;
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Tab => return ViewEvent::NextTab,
            KeyCode::BackTab => return ViewEvent::PrevTab,
            _ => {}
        }

        ViewEvent::None
    }

    fn is_icon_section(&self) -> bool {
        matches!(
            self.section,
            SettingsSection::Icons | SettingsSection::Stars | SettingsSection::Planets
        )
    }

    fn get_icon_count(&self, settings: &Settings) -> usize {
        match self.section {
            SettingsSection::Icons => settings.icons.len(),
            SettingsSection::Stars => settings.stars.len(),
            SettingsSection::Planets => settings.planets.len(),
            _ => 0,
        }
    }

    fn get_icon_keys(&self, settings: &Settings) -> Vec<String> {
        let icons: &HashMap<String, Icon> = match self.section {
            SettingsSection::Icons => &settings.icons,
            SettingsSection::Stars => &settings.stars,
            SettingsSection::Planets => &settings.planets,
            _ => &settings.icons,
        };
        let mut keys: Vec<String> = icons.keys().cloned().collect();
        keys.sort();
        keys
    }

    fn get_field_count(&self, settings: &Settings) -> usize {
        if self.is_icon_section() {
            return self.get_icon_count(settings) * 3;
        }
        self.get_fields(settings).len()
    }

    fn get_fields(&self, settings: &Settings) -> Vec<FieldInfo> {
        match self.section {
            SettingsSection::JournalReader => vec![
                FieldInfo {
                    label: "Journal Directory".to_string(),
                    field: EditableField::StringValue(settings.journal_reader.journal_directory.clone()),
                },
                FieldInfo {
                    label: "Action at Shutdown".to_string(),
                    field: EditableField::EnumValue(vec!["Exit", "Nothing", "Continue"]),
                },
            ],
            SettingsSection::GraphicsEditor => vec![
                FieldInfo {
                    label: "Graphics Directory".to_string(),
                    field: EditableField::StringValue(settings.graphics_editor.graphics_directory.clone()),
                },
            ],
            SettingsSection::Appearance => vec![
                FieldInfo {
                    label: "Color".to_string(),
                    field: EditableField::StringValue(settings.appearance.color.clone()),
                },
            ],
            SettingsSection::Explorer => vec![
                FieldInfo {
                    label: "Include System Name".to_string(),
                    field: EditableField::BoolValue(settings.explorer.include_system_name),
                },
            ],
            _ => vec![],
        }
    }

    fn get_icon_field_at(&self, focus: usize, settings: &Settings) -> Option<IconFieldInfo> {
        if !self.is_icon_section() {
            return None;
        }
        let icon_idx = focus / 3;
        let sub_idx = focus % 3;
        let keys = self.get_icon_keys(settings);
        if let Some(key) = keys.get(icon_idx) {
            let label = match sub_idx {
                0 => format!("  {}", key),
                1 => format!("  {}", key),
                2 => format!("  {}", key),
                _ => key.clone(),
            };
            Some(IconFieldInfo {
                name: key.clone(),
                sub_field: sub_idx,
                label,
            })
        } else {
            None
        }
    }

    fn apply_edit(&mut self, settings: &mut Settings) {
        let value = self.edit_buffer.clone();
        if self.is_icon_section() {
            let icon_idx = self.focus / 3;
            let sub_idx = self.focus % 3;
            let icon_keys = self.get_icon_keys(settings);
            if let Some(key) = icon_keys.get(icon_idx) {
                let icon = match self.section {
                    SettingsSection::Icons => settings.icons.get_mut(key),
                    SettingsSection::Stars => settings.stars.get_mut(key),
                    SettingsSection::Planets => settings.planets.get_mut(key),
                    _ => None,
                };
                if let Some(icon) = icon {
                    match sub_idx {
                        0 => icon.char = value,
                        1 => icon.color = value,
                        _ => {}
                    }
                }
            }
        } else {
            match self.section {
                SettingsSection::JournalReader => match self.focus {
                    0 => settings.journal_reader.journal_directory = value,
                    1 => {
                        if let Ok(action) = ActionAtShutdownSignal::from_str(&value) {
                            settings.journal_reader.action_at_shutdown_signal = action;
                        }
                    }
                    _ => {}
                },
                SettingsSection::GraphicsEditor => {
                    if self.focus == 0 {
                        settings.graphics_editor.graphics_directory = value;
                    }
                }
                SettingsSection::Appearance => {
                    if self.focus == 0 {
                        settings.appearance.color = value;
                    }
                }
                _ => {}
            }
        }
    }

    fn toggle_bool(&self, focus: usize, settings: &mut Settings) {
        if self.section == SettingsSection::Explorer && focus == 0 {
            settings.explorer.include_system_name = !settings.explorer.include_system_name;
        }
    }

    fn cycle_enum(&self, focus: usize, options: &[&'static str], settings: &mut Settings) {
        if self.section == SettingsSection::JournalReader && focus == 1 {
            let current = &settings.journal_reader.action_at_shutdown_signal;
            let current_str = current.to_string();
            let idx = options.iter().position(|o| *o == current_str.as_str()).unwrap_or(0);
            let next_idx = (idx + 1) % options.len();
            if let Ok(action) = ActionAtShutdownSignal::from_str(options[next_idx]) {
                settings.journal_reader.action_at_shutdown_signal = action;
            }
        }
    }

    fn edit_icon_field(&mut self, settings: &mut Settings) {
        let icon_idx = self.focus / 3;
        let sub_idx = self.focus % 3;
        let icon_keys = self.get_icon_keys(settings);
        if let Some(key) = icon_keys.get(icon_idx) {
            if sub_idx == 2 {
                let icon = match self.section {
                    SettingsSection::Icons => settings.icons.get_mut(key),
                    SettingsSection::Stars => settings.stars.get_mut(key),
                    SettingsSection::Planets => settings.planets.get_mut(key),
                    _ => None,
                };
                if let Some(icon) = icon {
                    icon.enabled = !icon.enabled;
                }
            } else {
                self.editing = true;
                let icon = match self.section {
                    SettingsSection::Icons => settings.icons.get(key),
                    SettingsSection::Stars => settings.stars.get(key),
                    SettingsSection::Planets => settings.planets.get(key),
                    _ => None,
                };
                if let Some(icon) = icon {
                    match sub_idx {
                        0 => self.edit_buffer = icon.char.clone(),
                        1 => self.edit_buffer = icon.color.clone(),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, settings: &Settings) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .split(area);

        self.render_sidebar(frame, chunks[0]);
        self.render_content(frame, chunks[1], settings);
    }

    fn render_sidebar(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = SettingsSection::all()
            .iter()
            .map(|s| ListItem::new(s.title()))
            .collect();

        let selected_idx = SettingsSection::all()
            .iter()
            .position(|s| *s == self.section)
            .unwrap_or(0);

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" Sections (a/d) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = ratatui::widgets::ListState::default().with_selected(Some(selected_idx));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect, settings: &Settings) {
        if self.is_icon_section() {
            self.render_icon_content(frame, area, settings);
        } else {
            self.render_regular_content(frame, area, settings);
        }
    }

    fn render_regular_content(&self, frame: &mut Frame, area: Rect, settings: &Settings) {
        let fields = self.get_fields(settings);
        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                self.section.title(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Space to edit, Enter to confirm, Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        for (i, field) in fields.iter().enumerate() {
            let is_focused = i == self.focus;
            let is_editing = self.editing && is_focused;

            let value_str = if is_editing {
                format!("{}_", self.edit_buffer)
            } else {
                match &field.field {
                    EditableField::StringValue(s) => s.clone(),
                    EditableField::BoolValue(b) => if *b { "true" } else { "false" }.to_string(),
                    EditableField::EnumValue(_) => {
                        if self.section == SettingsSection::JournalReader && i == 1 {
                            settings.journal_reader.action_at_shutdown_signal.to_string()
                        } else {
                            String::new()
                        }
                    }
                }
            };

            let line = if is_editing {
                Line::from(vec![
                    Span::styled(
                        format!(" > {:<25} ", field.label),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("= [ {} ]", value_str),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else if is_focused {
                Line::from(vec![
                    Span::styled(
                        format!("   {:<25} ", field.label),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("= {}", value_str),
                        Style::default().fg(Color::Cyan),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("   {:<25} ", field.label),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("= {}", value_str),
                        Style::default().fg(Color::Gray),
                    ),
                ])
            };
            lines.push(line);
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} (w/s: navigate, space: edit) ", self.section.title()))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_icon_content(&mut self, _frame: &mut Frame, area: Rect, settings: &Settings) {
        let icon_keys = self.get_icon_keys(settings);
        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                self.section.title(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Space to edit, Enter to confirm, Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("{:<35} {:<8} {:<15} {:<8}", "Name", "Char", "Color", "Enabled"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "-".repeat(70),
                Style::default().fg(Color::DarkGray),
            )),
        ];

        for (list_idx, name) in icon_keys.iter().enumerate() {
            let char_focus = self.focus == list_idx * 3;
            let color_focus = self.focus == list_idx * 3 + 1;
            let enabled_focus = self.focus == list_idx * 3 + 2;

            let icon = match self.section {
                SettingsSection::Icons => settings.icons.get(name),
                SettingsSection::Stars => settings.stars.get(name),
                SettingsSection::Planets => settings.planets.get(name),
                _ => None,
            };

            if let Some(icon) = icon {
                let char_display = if self.editing && self.focus / 3 == list_idx && self.focus % 3 == 0 {
                    format!("{}_", self.edit_buffer)
                } else {
                    icon.char.clone()
                };
                let color_display = if self.editing && self.focus / 3 == list_idx && self.focus % 3 == 1 {
                    format!("{}_", self.edit_buffer)
                } else {
                    icon.color.clone()
                };
                let enabled_display = if icon.enabled { "Yes" } else { "No" };

                let char_style = if char_focus {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let color_style = if color_focus {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let enabled_style = if enabled_focus {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if icon.enabled {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };
                let name_style = if char_focus || color_focus || enabled_focus {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let line = Line::from(vec![
                    Span::styled(format!("{:<35} ", name), name_style),
                    Span::styled(format!("{:<8} ", char_display), char_style),
                    Span::styled(format!("{:<15} ", color_display), color_style),
                    Span::styled(enabled_display, enabled_style),
                ]);
                lines.push(line);
            }
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} (w/s: navigate, space: edit) ", self.section.title()))
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .wrap(Wrap { trim: false });

        _frame.render_widget(paragraph, area);
    }

    pub fn save_settings(&self, settings: &Settings) {
        let json = serde_json::to_string_pretty(settings).expect("Failed to serialize settings");
        let settings_path = std::env::var("HOME")
            .map(|home| format!("{}/.config/edcas-client/settings.json", home))
            .unwrap_or_else(|_| "settings.json".to_string());

        if let Some(parent) = std::path::Path::new(&settings_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&settings_path, json) {
            eprintln!("Failed to save settings to {}: {}", settings_path, e);
        }
    }
}
