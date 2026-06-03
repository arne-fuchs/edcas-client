use crate::event_shim::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::str::FromStr;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;
use tracing::{info, warn, error};

#[cfg(not(target_arch = "wasm32"))]
use crate::journal_reader::{find_latest_journal_file, start_bulk_upload, BulkUploadProgress};

use crate::settings::Settings;
use crate::settings::icons::Icon;
use crate::settings::journal_reader::ActionAtShutdownSignal;
use crate::views::ViewEvent;

#[derive(Default, Clone, Copy, PartialEq)]
enum SettingsSection {
    #[default]
    JournalReader,
    Appearance,
    Explorer,
    Icons,
    Stars,
    Planets,
    About,
}

impl SettingsSection {
    fn all() -> Vec<Self> {
        vec![
            SettingsSection::JournalReader,
            SettingsSection::Appearance,
            SettingsSection::Explorer,
            SettingsSection::Icons,
            SettingsSection::Stars,
            SettingsSection::Planets,
            SettingsSection::About,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            SettingsSection::JournalReader => "Journal Reader",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Explorer => "Explorer",
            SettingsSection::Icons => "Icons",
            SettingsSection::Stars => "Stars",
            SettingsSection::Planets => "Planets",
            SettingsSection::About => "About",
        }
    }
}

#[derive(Clone, PartialEq)]
enum CellType {
    /// A non-interactive group heading that starts a new "paragraph" of related settings.
    /// Navigation skips over these and they have no editable value.
    Header(&'static str),
    Label(String),
    StringValue(String),
    BoolValue(bool),
    ToggleEnabled(bool),
    Button(&'static str),
}

struct GridRow {
    cells: Vec<CellType>,
}

/// Rank of a star class key for display ordering: the Harvard spectral sequence (O B A F G K M,
/// hot→cool), then the cool brown-dwarf classes, giants/supergiants, proto stars, Wolf-Rayet,
/// carbon, S-type, white dwarfs, compact remnants and finally non-stellar bodies. Keys not in
/// the table rank last and fall back to alphabetical order, so unknown future classes still sort
/// deterministically.
fn star_class_rank(key: &str) -> usize {
    const ORDER: &[&str] = &[
        // Main sequence, hot → cool
        "O", "B", "A", "F", "G", "K", "M",
        // Cool dwarfs
        "L", "T", "Y",
        // Giants & supergiants
        "A_BlueWhiteSuperGiant", "F_WhiteSuperGiant", "K_OrangeGiant", "M_RedGiant",
        "M_RedSuperGiant",
        // Proto stars
        "TTS", "AeBe",
        // Wolf-Rayet
        "W", "WN", "WNC", "WC", "WO",
        // Carbon stars
        "C", "CN", "CJ", "CH", "CHd", "CS",
        // S-type
        "S", "MS", "MS S",
        // White dwarfs
        "D", "DA", "DAB", "DAV", "DAZ", "DAO", "DB", "DBV", "DBZ", "DC", "DCV", "DO", "DOV",
        "DQ", "DX",
        // Compact remnants
        "N", "H", "SupermassiveBlackHole",
        // Non-stellar / exotic
        "X", "RoguePlanet", "Nebula", "StellarRemnantNebula",
    ];
    ORDER.iter().position(|&c| c == key).unwrap_or(usize::MAX)
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
    #[cfg(not(target_arch = "wasm32"))]
    bulk_upload_rx: Option<mpsc::Receiver<BulkUploadProgress>>,
    #[cfg(not(target_arch = "wasm32"))]
    bulk_upload_progress: Option<BulkUploadProgress>,
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
            #[cfg(not(target_arch = "wasm32"))]
            bulk_upload_rx: None,
            #[cfg(not(target_arch = "wasm32"))]
            bulk_upload_progress: None,
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
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Tab => {
                let sections = SettingsSection::all();
                self.sidebar_focus = (self.sidebar_focus + 1) % sections.len();
                self.section = sections[self.sidebar_focus];
                self.reset_content_position(settings);
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') => match self.focus {
                FocusArea::Sidebar => {
                    if self.sidebar_focus > 0 {
                        self.sidebar_focus -= 1;
                        self.section = SettingsSection::all()[self.sidebar_focus];
                        self.reset_content_position(settings);
                    }
                }
                FocusArea::Content => {
                    self.move_to_prev_selectable(settings);
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
                    self.move_to_next_selectable(settings);
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
            KeyCode::Char(' ') | KeyCode::Enter => {
                if matches!(self.focus, FocusArea::Content) {
                    let grid = self.build_grid(settings);
                    if self.row < grid.len() && self.col < grid[self.row].cells.len() {
                        let cell = grid[self.row].cells[self.col].clone();
                        match cell {
                            CellType::StringValue(ref s) => {
                                self.editing = true;
                                self.edit_buffer = s.clone();
                            }
                            CellType::BoolValue(_) => {
                                self.toggle_bool(settings);
                                return ViewEvent::SettingsChanged;
                            }
                            CellType::ToggleEnabled(_) => {
                                self.toggle_enabled(settings);
                                return ViewEvent::SettingsChanged;
                            }
                            CellType::Button("[ Open in default app ]") => {
                                #[cfg(not(target_arch = "wasm32"))]
                                self.open_current_log(settings);
                            }
                            CellType::Button("[ Upload All Journal Logs ]") => {
                                #[cfg(not(target_arch = "wasm32"))]
                                self.start_bulk_upload(settings);
                            }
                            CellType::Button(_) => {}
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }

        ViewEvent::None
    }

    fn reset_content_position(&mut self, settings: &Settings) {
        self.row = 0;
        self.col = 0;
        self.content_scroll = 0;
        // The first row may be a group header; land on the first selectable row instead.
        if !self.row_selectable(self.row, settings) {
            self.move_to_next_selectable(settings);
        }
        self.clamp_col(settings);
    }

    /// A row is selectable unless its first cell is a [`CellType::Header`] (a group heading).
    fn row_selectable(&self, idx: usize, settings: &Settings) -> bool {
        let grid = self.build_grid(settings);
        !matches!(
            grid.get(idx).and_then(|r| r.cells.first()),
            Some(CellType::Header(_))
        )
    }

    /// Moves the cursor up to the nearest selectable row, skipping headers.
    fn move_to_prev_selectable(&mut self, settings: &Settings) {
        let mut i = self.row;
        while i > 0 {
            i -= 1;
            if self.row_selectable(i, settings) {
                self.row = i;
                self.clamp_col(settings);
                self.ensure_scroll_visible();
                return;
            }
        }
    }

    /// Moves the cursor down to the nearest selectable row, skipping headers.
    fn move_to_next_selectable(&mut self, settings: &Settings) {
        let row_count = self.build_grid(settings).len();
        let mut i = self.row + 1;
        while i < row_count {
            if self.row_selectable(i, settings) {
                self.row = i;
                self.clamp_col(settings);
                self.ensure_scroll_visible();
                return;
            }
            i += 1;
        }
    }

    /// Label of the row the cursor is on, if its first cell is a [`CellType::Label`]. Used by
    /// the edit/toggle handlers so they key off the setting's name rather than a row index
    /// (which shifts when rows are regrouped or headers are inserted).
    fn row_label(&self, settings: &Settings) -> Option<String> {
        let grid = self.build_grid(settings);
        match grid.get(self.row).and_then(|r| r.cells.first()) {
            Some(CellType::Label(s)) => Some(s.clone()),
            _ => None,
        }
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
        if self.section == SettingsSection::Stars {
            // Order by the Harvard spectral sequence rather than alphabetically, with the key
            // name as a stable tie-breaker for any classes sharing a rank.
            keys.sort_by(|a, b| {
                star_class_rank(a)
                    .cmp(&star_class_rank(b))
                    .then_with(|| a.cmp(b))
            });
        } else {
            keys.sort();
        }
        keys
    }

    fn build_grid(&self, settings: &Settings) -> Vec<GridRow> {
        if self.section == SettingsSection::About {
            return Vec::new();
        }
        if self.is_icon_section() {
            self.build_icon_grid(settings)
        } else {
            self.build_regular_grid(settings)
        }
    }

    fn build_regular_grid(&self, settings: &Settings) -> Vec<GridRow> {
        match self.section {
            SettingsSection::JournalReader => vec![
                // ── Journal Reader ──────────────────────────────────────
                GridRow {
                    cells: vec![
                        CellType::Label("Journal Directory".to_string()),
                        CellType::StringValue(settings.journal_reader.journal_directory.clone()),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Current Log File".to_string()),
                        CellType::Button("[ Open in default app ]"),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Action at Shutdown".to_string()),
                        CellType::StringValue(settings.journal_reader.action_at_shutdown_signal.to_string()),
                    ],
                },
                // ── edcas API ───────────────────────────────────────────
                GridRow {
                    cells: vec![CellType::Header("edcas API")],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("edcas API URL".to_string()),
                        CellType::StringValue(settings.api_url.clone()),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Send to edcas API".to_string()),
                        CellType::BoolValue(settings.edcas_api_enabled),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Upload History".to_string()),
                        CellType::Button("[ Upload All Journal Logs ]"),
                    ],
                },
                // ── EDDN ────────────────────────────────────────────────
                GridRow {
                    cells: vec![CellType::Header("EDDN")],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("Send to EDDN".to_string()),
                        CellType::BoolValue(settings.eddn_enabled),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("EDDN URL".to_string()),
                        CellType::StringValue(settings.eddn_url.clone()),
                    ],
                },
                GridRow {
                    cells: vec![
                        CellType::Label("EDDN Test Mode".to_string()),
                        CellType::BoolValue(settings.eddn_test_mode),
                    ],
                },
            ],
            SettingsSection::Appearance => vec![GridRow {
                cells: vec![
                    CellType::Label("Accent Color (name, #rrggbb, or r,g,b)".to_string()),
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
                        1 => {
                            info!("Updated icon char for '{}': '{}'", key, value);
                            icon.char = value;
                        }
                        2 => {
                            info!("Updated icon color for '{}': '{}'", key, value);
                            icon.color = value;
                        }
                        _ => {}
                    }
                }
            }
        } else {
            match self.section {
                SettingsSection::JournalReader => match self.row_label(settings).as_deref() {
                    Some("Journal Directory") => {
                        info!("Updated journal directory: '{}'", value);
                        settings.journal_reader.journal_directory = value;
                    }
                    Some("Action at Shutdown") => {
                        if let Ok(action) = ActionAtShutdownSignal::from_str(&value) {
                            info!("Updated shutdown action: {}", action);
                            settings.journal_reader.action_at_shutdown_signal = action;
                        } else {
                            warn!("Invalid shutdown action value: '{}'", value);
                        }
                    }
                    Some("edcas API URL") => {
                        info!("Updated edcas API URL: '{}'", value);
                        settings.api_url = value;
                    }
                    Some("EDDN URL") => {
                        info!("Updated EDDN URL: '{}'", value);
                        settings.eddn_url = value;
                    }
                    _ => {}
                },
                SettingsSection::Appearance => {
                    if self.row == 0 {
                        info!("Updated appearance color: '{}'", value);
                        settings.appearance.color = value;
                    }
                }
                _ => {}
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_bulk_upload(&mut self, settings: &Settings) {
        let Some(api_url) = settings.edcas_api_url() else {
            warn!("edcas API upload disabled or URL not configured — cannot upload journal history");
            return;
        };
        let dir = settings.journal_reader.journal_directory.trim().to_string();
        if dir.is_empty() {
            warn!("Journal directory not configured");
            return;
        }
        info!("Starting bulk journal upload from: {}", dir);
        let rx = start_bulk_upload(PathBuf::from(dir), api_url);
        self.bulk_upload_rx = Some(rx);
        self.bulk_upload_progress = None;
    }

    pub fn poll_bulk_upload(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref rx) = self.bulk_upload_rx {
            // Drain all pending updates, keeping the last one.
            let mut last = None;
            while let Ok(p) = rx.try_recv() {
                last = Some(p);
            }
            if let Some(p) = last {
                let done = p.done || p.error.is_some();
                self.bulk_upload_progress = Some(p);
                if done {
                    self.bulk_upload_rx = None;
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_current_log(&self, settings: &Settings) {
        let dir = &settings.journal_reader.journal_directory;
        if dir.is_empty() {
            warn!("Journal directory not configured");
            return;
        }
        match find_latest_journal_file(&PathBuf::from(dir)) {
            Some(path) => {
                info!("Opening log file: {}", path.display());
                if let Err(e) = opener::open(&path) {
                    error!("Failed to open log file: {}", e);
                }
            }
            None => {
                warn!("No journal log file found in: {}", dir);
            }
        }
    }

    /// Toggles the boolean setting at the focused section/row.
    fn toggle_bool(&self, settings: &mut Settings) {
        match self.section {
            SettingsSection::Explorer => {
                settings.explorer.include_system_name = !settings.explorer.include_system_name;
                info!("Toggled include_system_name: {}", settings.explorer.include_system_name);
            }
            SettingsSection::JournalReader => match self.row_label(settings).as_deref() {
                Some("Send to edcas API") => {
                    settings.edcas_api_enabled = !settings.edcas_api_enabled;
                    info!("Toggled edcas API upload: {}", settings.edcas_api_enabled);
                }
                Some("Send to EDDN") => {
                    settings.eddn_enabled = !settings.eddn_enabled;
                    info!("Toggled EDDN upload: {}", settings.eddn_enabled);
                }
                Some("EDDN Test Mode") => {
                    settings.eddn_test_mode = !settings.eddn_test_mode;
                    info!("Toggled EDDN test mode: {}", settings.eddn_test_mode);
                }
                _ => {}
            },
            _ => {}
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
                info!("Toggled icon '{}' enabled: {}", key, icon.enabled);
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
                            .bg(crate::theme::accent())
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    ListItem::new(format!("  {}", s.title()))
                }
            })
            .collect();

        let sidebar_title = if matches!(self.focus, FocusArea::Sidebar) {
            " Sections (Tab: next, w/s: nav, d: content) "
        } else {
            " Sections (Tab: next, a: sidebar) "
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(sidebar_title)
                    .borders(Borders::ALL)
                    .style(
                        if matches!(self.focus, FocusArea::Sidebar) {
                            Style::default().fg(crate::theme::accent())
                        } else {
                            Style::default().fg(Color::DarkGray)
                        }
                    ),
            );

        let mut state = ratatui::widgets::ListState::default().with_selected(Some(self.sidebar_focus));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect, settings: &Settings) {
        // Reserve 3 rows at the bottom for the progress bar when uploading.
        #[cfg(not(target_arch = "wasm32"))]
        let (content_area, progress_area) = if self.bulk_upload_progress.is_some() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };
        #[cfg(target_arch = "wasm32")]
        let (content_area, progress_area): (Rect, Option<Rect>) = (area, None);

        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(area), Some(ref prog)) = (progress_area, &self.bulk_upload_progress) {
            let ratio = if prog.total_files == 0 {
                0.0
            } else {
                prog.current_file as f64 / prog.total_files as f64
            };
            let label = if prog.done {
                format!("Done — {} files, {} lines uploaded", prog.total_files, prog.lines_done)
            } else if let Some(ref e) = prog.error {
                format!("Error: {e}")
            } else {
                format!(
                    "Uploading: file {}/{} — {} lines",
                    prog.current_file, prog.total_files, prog.lines_done
                )
            };
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(crate::theme::accent()).bg(Color::DarkGray))
                .ratio(ratio)
                .label(label);
            frame.render_widget(gauge, area);
        }
        let _ = progress_area;

        let area = content_area;

        if self.section == SettingsSection::About {
            let about_lines = super::about::build_lines();
            let inner_h = area.height.saturating_sub(2) as usize;
            let max_scroll = about_lines.len().saturating_sub(inner_h);
            let scroll = self.row.min(max_scroll) as u16;
            frame.render_widget(
                ratatui::widgets::Paragraph::new(about_lines)
                    .block(ratatui::widgets::Block::default()
                        .title(" About ")
                        .borders(ratatui::widgets::Borders::ALL)
                        .border_style(Style::default().fg(crate::theme::accent())))
                    .scroll((scroll, 0)),
                area,
            );
            return;
        }

        let grid = self.build_grid(settings);
        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                self.section.title(),
                Style::default()
                    .fg(crate::theme::accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                if matches!(self.focus, FocusArea::Content) {
                    "Tab: next section | w/s: rows | a/d: columns/panel | space: edit | enter: save | esc: cancel"
                } else {
                    "Tab: next section | d: focus content | w/s: navigate sections"
                },
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        for (row_idx, row) in grid.iter().enumerate() {
            // A group header starts a new paragraph: a blank spacer line followed by a bold
            // heading. It is not selectable, so it never shows the focus cursor.
            if let Some(CellType::Header(title)) = row.cells.first() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    title.to_string(),
                    Style::default()
                        .fg(crate::theme::accent())
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }

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
                    CellType::Header(s) => s.to_string(),
                    CellType::Label(s) => s.clone(),
                    CellType::StringValue(s) => {
                        if is_editing {
                            format!("{}_", self.edit_buffer)
                        } else {
                            s.clone()
                        }
                    }
                    CellType::BoolValue(b) => if *b { "true" } else { "false" }.to_string(),
                    CellType::ToggleEnabled(b) => if *b { "Yes" } else { "No" }.to_string(),
                    CellType::Button(label) => label.to_string(),
                };

                let style = if is_editing {
                    Style::default()
                        .fg(Color::Black)
                        .bg(crate::theme::accent())
                        .add_modifier(Modifier::BOLD)
                } else if is_cell_focused {
                    Style::default()
                        .fg(Color::Black)
                        .bg(crate::theme::accent())
                        .add_modifier(Modifier::BOLD)
                } else if matches!(cell, CellType::Label(_)) {
                    Style::default().fg(Color::Cyan)
                } else if matches!(cell, CellType::ToggleEnabled(true)) {
                    Style::default().fg(Color::Green)
                } else if matches!(cell, CellType::ToggleEnabled(false)) {
                    Style::default().fg(Color::Red)
                } else if matches!(cell, CellType::Button(_)) {
                    Style::default().fg(crate::theme::accent())
                } else {
                    Style::default().fg(Color::White)
                };

                let padding = match cell {
                    CellType::Label(_) => 35,
                    CellType::Button(_) => 0,
                    CellType::StringValue(_) => 12,
                    _ => 10,
                };
                row_spans.push(Span::styled(format!("{:<padding$}", display), style));
            }

            lines.push(Line::from(row_spans));
        }

        let content_border_style = if matches!(self.focus, FocusArea::Content) {
            Style::default().fg(crate::theme::accent())
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
        #[cfg(target_arch = "wasm32")]
        {
            settings.save_wasm();
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let json = serde_json::to_string_pretty(settings).expect("Failed to serialize settings");
            let settings_path = crate::settings::config_dir().join("settings.json");

            if let Some(parent) = settings_path.parent() {
                let _ = std::fs::create_dir_all(parent);
                let tmp = parent.join("settings.json.tmp");
                if std::fs::write(&tmp, &json).is_ok() {
                    match std::fs::rename(&tmp, &settings_path) {
                        Ok(_) => tracing::info!("Settings saved to {}", settings_path.display()),
                        Err(e) => tracing::error!("Failed to save settings to {}: {}", settings_path.display(), e),
                    }
                } else {
                    tracing::error!("Failed to write settings tmp file");
                }
            }
        }
    }
}
