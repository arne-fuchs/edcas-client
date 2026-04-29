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

use crate::settings::Settings;
use crate::settings::icons::Icon;
use crate::settings::journal_reader::ActionAtShutdownSignal;
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
enum CellType {
    Label(String),
    StringValue(String),
    BoolValue(bool),
    EnumValue(Vec<&'static str>),
    ToggleEnabled(bool),
}

impl CellType {
    fn is_editable(&self) -> bool {
        matches!(
            self,
            CellType::StringValue(_)
                | CellType::BoolValue(_)
                | CellType::EnumValue(_)
                | CellType::ToggleEnabled(_)
        )
    }
}

struct GridRow {
    cells: Vec<CellType>,
}

pub struct SettingsView {
    section: SettingsSection,
    sidebar_focus: usize,
    row: usize,
    col: usize,
    editing: bool,
    edit_buffer: String,
    content_scroll: usize,
    focus: FocusArea,
}

enum FocusArea {
    Sidebar,
    Content,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            section: SettingsSection::default(),
            sidebar_focus: 0,
            row: 0,
            col: 0,
            editing: false,
            edit_buffer: String::new(),
            content_scroll: 0,
            focus: FocusArea::Sidebar,
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

        match key.code {
            KeyCode::Char('w') => match self.focus {
                FocusArea::Sidebar => {
                    if self.sidebar_focus > 0 {
                        self.sidebar_focus -= 1;
                        self.section = SettingsSection::all()[self.sidebar_focus];
                        self.reset_content_position(settings);
                    }
                }
                FocusArea::Content => {
                    if self.row > 0 {
                        self.row -= 1;
                        self.clamp_col(settings);
                        self.ensure_scroll_visible();
                    }
                }
            },
            KeyCode::Char('s') => match self.focus {
                FocusArea::Sidebar => {
                    let section_count = SettingsSection::all().len();
                    if self.sidebar_focus + 1 < section_count {
                        self.sidebar_focus += 1;
                        self.section = SettingsSection::all()[self.sidebar_focus];
                        self.reset_content_position(settings);
                    }
                }
                FocusArea::Content => {
                    let grid = self.build_grid(settings);
                    let row_count = grid.len();
                    if self.row + 1 < row_count {
                        self.row += 1;
                        self.clamp_col(settings);
                        self.ensure_scroll_visible();
                    }
                }
            },
            KeyCode::Char('a') => match self.focus {
                FocusArea::Sidebar => {}
                FocusArea::Content => {
                    if self.col > 0 {
                        self.col -= 1;
                    } else {
                        self.focus = FocusArea::Sidebar;
                    }
                }
            },
            KeyCode::Char('d') => match self.focus {
                FocusArea::Sidebar => {
                    self.focus = FocusArea::Content;
                }
                FocusArea::Content => {
                    let grid = self.build_grid(settings);
                    let max_col = if self.row < grid.len() {
                        grid[self.row].cells.len()
                    } else {
                        0
                    };
                    if self.col + 1 < max_col {
                        self.col += 1;
                    }
                }
            },
            KeyCode::Char(' ') => {
                if matches!(self.focus, FocusArea::Content) {
                    let grid = self.build_grid(settings);
                    if self.row < grid.len() && self.col < grid[self.row].cells.len() {
                        let cell = &grid[self.row].cells[self.col];
                        match cell {
                            CellType::StringValue(_) => {
                                self.editing = true;
                                self.edit_buffer = match cell {
                                    CellType::StringValue(s) => s.clone(),
                                    _ => String::new(),
                                };
                            }
                            CellType::BoolValue(_) => {
                                settings.explorer.include_system_name = !settings.explorer.include_system_name;
                                return ViewEvent::SettingsChanged;
                            }
                            CellType::ToggleEnabled(_) => {
                                self.toggle_enabled(settings);
                                return ViewEvent::SettingsChanged;
                            }
                            _ => {}
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

    fn reset_content_position(&mut self, settings: &Settings) {
        self.row = 0;
        self.col = 0;
        self.content_scroll = 0;
        self.clamp_col(settings);
    }

    fn clamp_col(&mut self, settings: &Settings) {
        let grid = self.build_grid(settings);
        if self.row < grid.len() {
            let max_col = grid[self.row].cells.len();
            if self.col >= max_col {
                self.col = max_col.saturating_sub(1);
            }
        }
    }

    fn ensure_scroll_visible(&mut self) {
        let visible_lines = 15;
        if self.row >= self.content_scroll + visible_lines {
            self.content_scroll = self.row.saturating_sub(visible_lines.saturating_sub(1));
        } else if self.row < self.content_scroll {
            self.content_scroll = self.row;
        }
    }

    fn is_icon_section(&self) -> bool {
        matches!(
            self.section,
            SettingsSection::Icons | SettingsSection::Stars | SettingsSection::Planets
        )
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

    fn build_grid(&self, settings: &Settings) -> Vec<GridRow> {
        if self.is_icon_section() {
            self.build_icon_grid(settings)
        } else {
            self.build_regular_grid(settings)
        }
    }

    fn build_regular_grid(&self, settings: &Settings) -> Vec<GridRow> {
        match self.section {
            SettingsSection::JournalReader => vec![
                GridRow {
                    cells: vec![
                        CellType::Label("Journal Directory".to_string()),
                        CellType::StringValue(settings.journal_reader.journal_directory.clone()),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Action at Shutdown".to_string()),
                        CellType::StringValue(settings.journal_reader.action_at_shutdown_signal.to_string()),
                    ],
                },
            ],
            SettingsSection::GraphicsEditor => vec![GridRow {
                cells: vec![
                    CellType::Label("Graphics Directory".to_string()),
                    CellType::StringValue(settings.graphics_editor.graphics_directory.clone()),
                ],
            }],
            SettingsSection::Appearance => vec![GridRow {
                cells: vec![
                    CellType::Label("Color".to_string()),
                    CellType::StringValue(settings.appearance.color.clone()),
                ],
            }],
            SettingsSection::Explorer => vec![GridRow {
                cells: vec![
                    CellType::Label("Include System Name".to_string()),
                    CellType::BoolValue(settings.explorer.include_system_name),
                ],
            }],
            _ => vec![],
        }
    }

    fn build_icon_grid(&self, settings: &Settings) -> Vec<GridRow> {
        let keys = self.get_icon_keys(settings);
        let icons: &HashMap<String, Icon> = match self.section {
            SettingsSection::Icons => &settings.icons,
            SettingsSection::Stars => &settings.stars,
            SettingsSection::Planets => &settings.planets,
            _ => &settings.icons,
        };

        keys.iter()
            .filter_map(|key| {
                icons.get(key).map(|icon| GridRow {
                    cells: vec![
                        CellType::Label(key.clone()),
                        CellType::StringValue(icon.char.clone()),
                        CellType::StringValue(icon.color.clone()),
                        CellType::ToggleEnabled(icon.enabled),
                    ],
                })
            })
            .collect()
    }

    fn apply_edit(&mut self, settings: &mut Settings) {
        let value = self.edit_buffer.clone();

        if self.is_icon_section() {
            let icon_keys = self.get_icon_keys(settings);
            if let Some(key) = icon_keys.get(self.row) {
                let icon = match self.section {
                    SettingsSection::Icons => settings.icons.get_mut(key),
                    SettingsSection::Stars => settings.stars.get_mut(key),
                    SettingsSection::Planets => settings.planets.get_mut(key),
                    _ => None,
                };
                if let Some(icon) = icon {
                    match self.col {
                        1 => icon.char = value,
                        2 => icon.color = value,
                        _ => {}
                    }
                }
            }
        } else {
            match self.section {
                SettingsSection::JournalReader => match self.row {
                    0 => settings.journal_reader.journal_directory = value,
                    1 => {
                        if let Ok(action) = ActionAtShutdownSignal::from_str(&value) {
                            settings.journal_reader.action_at_shutdown_signal = action;
                        }
                    }
                    _ => {}
                },
                SettingsSection::GraphicsEditor => {
                    if self.row == 0 {
                        settings.graphics_editor.graphics_directory = value;
                    }
                }
                SettingsSection::Appearance => {
                    if self.row == 0 {
                        settings.appearance.color = value;
                    }
                }
                _ => {}
            }
        }
    }

    fn toggle_enabled(&self, settings: &mut Settings) {
        let icon_keys = self.get_icon_keys(settings);
        if let Some(key) = icon_keys.get(self.row) {
            let icon = match self.section {
                SettingsSection::Icons => settings.icons.get_mut(key),
                SettingsSection::Stars => settings.stars.get_mut(key),
                SettingsSection::Planets => settings.planets.get_mut(key),
                _ => None,
            };
            if let Some(icon) = icon {
                icon.enabled = !icon.enabled;
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
            .enumerate()
            .map(|(i, s)| {
                let is_focused = i == self.sidebar_focus;
                if is_focused {
                    ListItem::new(Span::styled(
                        format!("> {}", s.title()),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    ListItem::new(format!("  {}", s.title()))
                }
            })
            .collect();

        let sidebar_title = if matches!(self.focus, FocusArea::Sidebar) {
            " Sections (w/s: nav, d: content) "
        } else {
            " Sections (a: sidebar) "
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(sidebar_title)
                    .borders(Borders::ALL)
                    .style(
                        if matches!(self.focus, FocusArea::Sidebar) {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }
                    ),
            );

        let mut state = ratatui::widgets::ListState::default().with_selected(Some(self.sidebar_focus));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect, settings: &Settings) {
        let grid = self.build_grid(settings);
        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                self.section.title(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if matches!(self.focus, FocusArea::Content) {
                    "w/s: rows | a/d: columns/panel | space: edit | enter: save | esc: cancel"
                } else {
                    "d: focus content | w/s: navigate sections"
                },
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        for (row_idx, row) in grid.iter().enumerate() {
            let is_row_focused = row_idx == self.row;
            let mut row_spans: Vec<Span> = Vec::new();

            row_spans.push(Span::styled(
                if is_row_focused { "> " } else { "  " },
                Style::default().fg(Color::DarkGray),
            ));

            for (col_idx, cell) in row.cells.iter().enumerate() {
                let is_cell_focused = is_row_focused && col_idx == self.col;
                let is_editing = self.editing && is_cell_focused;

                let display = match cell {
                    CellType::Label(s) => s.clone(),
                    CellType::StringValue(s) => {
                        if is_editing {
                            format!("{}_", self.edit_buffer)
                        } else {
                            s.clone()
                        }
                    }
                    CellType::BoolValue(b) => if *b { "true" } else { "false" }.to_string(),
                    CellType::EnumValue(_) => {
                        if self.section == SettingsSection::JournalReader && row_idx == 1 {
                            settings.journal_reader.action_at_shutdown_signal.to_string()
                        } else {
                            String::new()
                        }
                    }
                    CellType::ToggleEnabled(b) => if *b { "Yes" } else { "No" }.to_string(),
                };

                let style = if is_editing {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_cell_focused {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if matches!(cell, CellType::Label(_)) {
                    Style::default().fg(Color::Cyan)
                } else if matches!(cell, CellType::ToggleEnabled(true)) {
                    Style::default().fg(Color::Green)
                } else if matches!(cell, CellType::ToggleEnabled(false)) {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::White)
                };

                let padding = match cell {
                    CellType::Label(_) => 35,
                    CellType::StringValue(_) => 12,
                    _ => 10,
                };
                row_spans.push(Span::styled(format!("{:<padding$}", display), style));
            }

            lines.push(Line::from(row_spans));
        }

        let content_border_style = if matches!(self.focus, FocusArea::Content) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} ", self.section.title()))
                    .borders(Borders::ALL)
                    .style(content_border_style),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.content_scroll as u16, 0));

        frame.render_widget(paragraph, area);
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
