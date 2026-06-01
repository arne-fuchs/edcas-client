use crate::api_client::ApiClient;
use crate::event_shim::{KeyCode, KeyEvent};
use crate::views::ViewEvent;
#[cfg(not(target_arch = "wasm32"))]
use tracing::warn;
use edcas_common::api::{NearestCommodityQuery, NearestCommodityResult};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

const MAX_SUGGESTIONS: usize = 8;

static COMMODITY_NAMES: &[&str] = &[
    "Advanced Catalysers",
    "Advanced Medicines",
    "Agri-Medicines",
    "Ai Relics",
    "Alexandrite",
    "Algae",
    "Aluminium",
    "Animal Meat",
    "Animal Monitors",
    "Anomalous Bulk Scan Data",
    "Antique Jewellery",
    "Antiquities",
    "Aquaponic Systems",
    "Assault Plans",
    "Atmospheric Processors",
    "Basic Medicines",
    "Battle Weapons",
    "Bauxite",
    "Beer",
    "Benitoite",
    "Beryllium",
    "Bertrandite",
    "Bioreducing Lichen",
    "Biowaste",
    "Bismuth",
    "Black Box",
    "Bootleg Liquor",
    "Bromellite",
    "Building Fabricators",
    "Ceramic Composites",
    "Chemical Waste",
    "Classified Experimental Equipment",
    "Clothing",
    "CMM Composite",
    "Cobalt",
    "Coffee",
    "Coltan",
    "Combat Stabilisers",
    "Commercial Samples",
    "Computer Components",
    "Conductive Fabrics",
    "Consumer Technology",
    "Copper",
    "Crop Harvesters",
    "Cryolite",
    "Damaged Escape Pod",
    "Data Core",
    "Diplomatic Bag",
    "Domestic Appliances",
    "Emergency Power Cells",
    "Encrypted Correspondence",
    "Encrypted Data Storage",
    "Energy Grid Assembly",
    "Engineering Technical Drawings",
    "Evacuation Shelter",
    "Exhaust Manifold",
    "Experimental Chemicals",
    "Explosives",
    "Fish",
    "Food Cartridges",
    "Fossil Remnants",
    "Fruit And Vegetables",
    "Gallite",
    "Gallium",
    "Geological Equipment",
    "Gold",
    "Goslarite",
    "Grain",
    "Grand Prismatic Sapphire",
    "Hafnium 178",
    "Hazardous Environment Suits",
    "Heatsink Interlink",
    "HN Shock Mount",
    "Hydrogen Fuels",
    "Hydrogen Peroxide",
    "Imperial Slaves",
    "Improvised Components",
    "Indite",
    "Indium",
    "Industrial Firmware",
    "Insulating Membrane",
    "Intact Technology Package",
    "Ion Distributor",
    "Jadeite",
    "Kernite",
    "Land Enrichment Systems",
    "Landmines",
    "Large Survey Data Cache",
    "Lanthanum",
    "Leaked Installation Data",
    "Leather",
    "Lepidolite",
    "Liquid Oxygen",
    "Liquor",
    "Lithium",
    "Low Temperature Diamonds",
    "Magnetic Emitter Coil",
    "Marine Equipment",
    "Medical Diagnostic Equipment",
    "Methane Clathrate",
    "Methanol Monohydrate Crystals",
    "Microbial Furnaces",
    "Military Grade Fabrics",
    "Military Intelligence",
    "Military Plans",
    "Mineral Extractors",
    "Mineral Oil",
    "Modular Terminals",
    "Moissanite",
    "Monazite",
    "Muon Imager",
    "Musgravite",
    "Mysterious Idol",
    "Nanobreakers",
    "Narcotics",
    "Natural Fabrics",
    "Nerve Agents",
    "Non-Lethal Weapons",
    "Occupied Cryopod",
    "Osmium",
    "Painite",
    "Palladium",
    "Performance Enhancers",
    "Personal Effects",
    "Personal Weapons",
    "Pesticides",
    "Platinum",
    "Pod Core Tissue",
    "Pod Dead Tissue",
    "Pod Mesoglea",
    "Pod Outer Tissue",
    "Pod Shell Tissue",
    "Pod Surface Tissue",
    "Pod Tissue Sample",
    "Polymers",
    "Power Converter",
    "Power Generators",
    "Power Transfer Bus",
    "Praseodymium",
    "Precious Gems",
    "Progenitor Cells",
    "Prohibited Research Materials",
    "Prototype Tech",
    "Pyrophyllite",
    "Radiation Baffle",
    "Rare Artwork",
    "Reactive Armour",
    "Rebel Transmissions",
    "Reinforced Mounting Plate",
    "Resonating Separators",
    "Rhodplumsite",
    "Robotics",
    "Rutile",
    "Salvageable Wreckage",
    "Samarium",
    "Sap 8 Core Container",
    "Scientific Research",
    "Scientific Samples",
    "Scrap",
    "Semiconductors",
    "Serendibite",
    "Silver",
    "Skimmer Components",
    "Slaves",
    "Space Pioneer Relics",
    "Stimulants",
    "Structural Regulators",
    "Superconductors",
    "Surface Stabilisers",
    "Survival Equipment",
    "Synthetic Fabrics",
    "Synthetic Meat",
    "Synthetic Reagents",
    "Taaffeite",
    "Tactical Data",
    "Tantalum",
    "Tea",
    "Technical Blueprints",
    "Telemetry Suite",
    "Thallium",
    "Thermal Cooling Units",
    "Thorium",
    "Titanium",
    "Tobacco",
    "Toxic Waste",
    "Trade Data",
    "Tritium",
    "Unknown Artefact",
    "Unstable Data Core",
    "Uraninite",
    "Uranium",
    "Void Opals",
    "Water Purifiers",
    "Wine",
    "Wreckage Components",
];

/// Resolve a raw journal commodity key (e.g. `"$buildingfabricators_name;"`) to the
/// proper canonical name used by the API (e.g. `"Building Fabricators"`).
/// Strips `$`, `;`, `_Name`, `_name`, then matches against the known commodity list
/// by normalizing both sides to lowercase alphanumeric.
pub(super) fn resolve_commodity_canonical(raw: &str) -> String {
    let base = raw
        .trim_start_matches('$')
        .trim_end_matches(';')
        .trim_end_matches("_Name")
        .trim_end_matches("_name");
    let normalized = base.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
    COMMODITY_NAMES
        .iter()
        .find(|&&name| {
            let n = name.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
            n == normalized
        })
        .map(|&s| s.to_string())
        .unwrap_or_else(|| base.to_string())
}

#[derive(Clone, Copy, PartialEq)]
enum ActiveField {
    Commodity,
    System,
}

pub struct SearchNearestView {
    commodity_input: String,
    /// Canonical English name used for the API query (may differ from localized commodity_input)
    commodity_canonical: String,
    system_input: String,
    active_field: ActiveField,
    editing: bool,
    results: Vec<NearestCommodityResult>,
    selected_idx: usize,
    scroll: usize,
    status_msg: String,
    is_loading: bool,
    suggestions: Vec<&'static str>,
    suggestion_idx: Option<usize>,
    /// Minimum pad size filter: 'S' = no filter, 'M' = medium or large, 'L' = large only.
    pad_filter: char,
    #[cfg(not(target_arch = "wasm32"))]
    pending: Arc<Mutex<Option<Result<Vec<NearestCommodityResult>, String>>>>,
    #[cfg(target_arch = "wasm32")]
    pending: Rc<RefCell<Option<Vec<NearestCommodityResult>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<arboard::Clipboard>,
    #[cfg(target_arch = "wasm32")]
    clipboard: (),
}

impl SearchNearestView {
    pub fn new() -> Self {
        Self {
            commodity_input: String::new(),
            commodity_canonical: String::new(),
            system_input: String::new(),
            active_field: ActiveField::Commodity,
            editing: false,
            results: Vec::new(),
            selected_idx: 0,
            scroll: 0,
            status_msg: "Press Enter to start editing".into(),
            is_loading: false,
            suggestions: Vec::new(),
            suggestion_idx: None,
            pad_filter: 'S',
            #[cfg(not(target_arch = "wasm32"))]
            pending: Arc::new(Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            pending: Rc::new(RefCell::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: arboard::Clipboard::new().ok(),
            #[cfg(target_arch = "wasm32")]
            clipboard: (),
        }
    }

    pub fn prefill_and_search(&mut self, commodity: &str, canonical_name: &str, system: &str, ship_pad_size: char, api: &ApiClient) {
        self.commodity_input = commodity.to_string();
        self.commodity_canonical = canonical_name.to_string();
        self.system_input = system.to_string();
        self.pad_filter = ship_pad_size;
        self.active_field = ActiveField::Commodity;
        self.editing = false;
        self.do_search(api);
    }

    fn pad_matches(&self, r: &NearestCommodityResult) -> bool {
        match self.pad_filter {
            'L' => r.has_large_pad,
            'M' => r.has_large_pad || r.has_medium_pad,
            _ => true,
        }
    }

    fn normalize_for_match(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    }

    fn update_suggestions(&mut self) {
        if self.commodity_input.is_empty() {
            self.suggestions.clear();
            self.suggestion_idx = None;
            return;
        }
        let needle = Self::normalize_for_match(&self.commodity_input);
        self.suggestions = COMMODITY_NAMES
            .iter()
            .copied()
            .filter(|name| Self::normalize_for_match(name).contains(&needle))
            .take(MAX_SUGGESTIONS)
            .collect();
        if let Some(idx) = self.suggestion_idx {
            if idx >= self.suggestions.len() {
                self.suggestion_idx = None;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient) {
        if self.commodity_input.is_empty() {
            self.status_msg = "Enter a commodity name first".into();
            return;
        }
        self.is_loading = true;
        self.status_msg = format!(
            "Searching for '{}' near '{}'…",
            self.commodity_input, self.system_input
        );
        let pending = Arc::clone(&self.pending);
        let api_owned = api.clone();
        let commodity_query = if self.commodity_canonical.is_empty() {
            self.commodity_input.clone()
        } else {
            self.commodity_canonical.clone()
        };
        let query = NearestCommodityQuery {
            commodity: commodity_query,
            reference_system: self.system_input.clone(),
            limit: Some(10),
        };
        api.spawn(async move {
            let result = api_owned
                .search_nearest_commodity(&query)
                .await
                .map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient) {
        if self.commodity_input.is_empty() {
            self.status_msg = "Enter a commodity name first".into();
            return;
        }
        self.is_loading = true;
        self.status_msg = format!(
            "Searching for '{}' near '{}'…",
            self.commodity_input, self.system_input
        );
        let pending = Rc::clone(&self.pending);
        let api = api.clone();
        let query = NearestCommodityQuery {
            commodity: self.commodity_input.clone(),
            reference_system: self.system_input.clone(),
            limit: Some(10),
        };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_nearest_commodity(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_search(&mut self) {
        if let Some(result) = self.pending.lock().unwrap().take() {
            self.is_loading = false;
            match result {
                Ok(results) => {
                    self.results = results.into_iter().filter(|r| self.pad_matches(r)).collect();
                    let count = self.results.len();
                    self.selected_idx = 0;
                    self.scroll = 0;
                    let pad_note = match self.pad_filter {
                        'L' => " (large pad only)",
                        'M' => " (medium+ pad)",
                        _ => "",
                    };
                    self.status_msg = if count == 0 {
                        if self.system_input.is_empty() {
                            format!("No '{}' in stock anywhere{pad_note}", self.commodity_input)
                        } else {
                            format!(
                                "No '{}' found near '{}'{pad_note} — system may not be in DB",
                                self.commodity_input, self.system_input
                            )
                        }
                    } else {
                        format!("{count} station(s) found{pad_note}")
                    };
                }
                Err(e) => {
                    warn!(error = %e, "search_nearest_commodity error");
                    self.status_msg = format!("API error: {e}");
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending.borrow_mut().take() {
            self.is_loading = false;
            self.results = results.into_iter().filter(|r| self.pad_matches(r)).collect();
            let count = self.results.len();
            self.selected_idx = 0;
            self.scroll = 0;
            let pad_note = match self.pad_filter {
                'L' => " (large pad only)",
                'M' => " (medium+ pad)",
                _ => "",
            };
            self.status_msg = if count == 0 {
                format!("No '{}' found near '{}'{pad_note}", self.commodity_input, self.system_input)
            } else {
                format!("{count} station(s) found{pad_note}")
            };
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient) -> ViewEvent {
        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    if let Some(idx) = self.suggestion_idx {
                        if let Some(&name) = self.suggestions.get(idx) {
                            self.commodity_input = name.to_string();
                        }
                    }
                    self.editing = false;
                    self.suggestions.clear();
                    self.suggestion_idx = None;
                    self.do_search(api);
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.suggestions.clear();
                    self.suggestion_idx = None;
                }
                KeyCode::Tab => {
                    if self.active_field == ActiveField::Commodity && !self.suggestions.is_empty() {
                        let idx = self.suggestion_idx.unwrap_or(0);
                        if let Some(&name) = self.suggestions.get(idx) {
                            self.commodity_input = name.to_string();
                        }
                        self.suggestions.clear();
                        self.suggestion_idx = None;
                        self.active_field = ActiveField::System;
                    } else {
                        self.suggestions.clear();
                        self.suggestion_idx = None;
                        self.active_field = match self.active_field {
                            ActiveField::Commodity => ActiveField::System,
                            ActiveField::System => ActiveField::Commodity,
                        };
                    }
                }
                KeyCode::Down => {
                    if self.active_field == ActiveField::Commodity && !self.suggestions.is_empty() {
                        let next = self.suggestion_idx.map_or(0, |i| (i + 1).min(self.suggestions.len() - 1));
                        self.suggestion_idx = Some(next);
                    }
                }
                KeyCode::Up => {
                    if self.active_field == ActiveField::Commodity && !self.suggestions.is_empty() {
                        self.suggestion_idx = match self.suggestion_idx {
                            Some(0) | None => None,
                            Some(i) => Some(i - 1),
                        };
                    }
                }
                KeyCode::Backspace => match self.active_field {
                    ActiveField::Commodity => {
                        self.commodity_input.pop();
                        self.commodity_canonical.clear();
                        self.update_suggestions();
                    }
                    ActiveField::System => { self.system_input.pop(); }
                },
                KeyCode::Char(c) => match self.active_field {
                    ActiveField::Commodity => {
                        self.commodity_input.push(c);
                        self.commodity_canonical.clear();
                        self.suggestion_idx = None;
                        self.update_suggestions();
                    }
                    ActiveField::System => self.system_input.push(c),
                },
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Enter => {
                self.editing = true;
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => {
                self.selected_idx = self.selected_idx.saturating_sub(1);
                if self.selected_idx < self.scroll {
                    self.scroll = self.selected_idx;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_idx + 1 < self.results.len() {
                    self.selected_idx += 1;
                }
            }
            KeyCode::Char('c') => {
                if let Some(r) = self.results.get(self.selected_idx) {
                    let name = r.system_name.trim().to_string();
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(cb) = self.clipboard.as_mut() {
                        let _ = cb.set_text(&name);
                        self.status_msg = format!("Copied: {name}");
                    }
                }
                return ViewEvent::Consumed;
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let n_sugg = if self.editing && self.active_field == ActiveField::Commodity {
            self.suggestions.len()
        } else {
            0
        };
        let input_height = (6 + n_sugg) as u16;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(input_height), Constraint::Min(0)])
            .split(area);

        let commodity_style = if self.active_field == ActiveField::Commodity {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let system_style = if self.active_field == ActiveField::System {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let cursor_c = if self.editing && self.active_field == ActiveField::Commodity {
            "_"
        } else {
            ""
        };
        let cursor_s = if self.editing && self.active_field == ActiveField::System {
            "_"
        } else {
            ""
        };

        let status_text = if self.is_loading {
            "  Searching…".to_string()
        } else {
            format!("  {}", self.status_msg)
        };

        let mut input_lines = vec![
            Line::from(vec![
                Span::styled("  Commodity : ", Style::default().fg(Color::Cyan)),
                Span::styled(self.commodity_input.clone(), commodity_style),
                Span::styled(cursor_c, Style::default().fg(Color::Yellow)),
            ]),
        ];

        if self.editing && self.active_field == ActiveField::Commodity {
            for (i, &name) in self.suggestions.iter().enumerate() {
                let is_selected = self.suggestion_idx == Some(i);
                let (fg, bg) = if is_selected {
                    (Color::Black, Color::Rgb(255, 140, 0))
                } else {
                    (Color::DarkGray, Color::Reset)
                };
                input_lines.push(Line::from(vec![
                    Span::styled(
                        "  \u{2502} ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{:<}", name),
                        Style::default().fg(fg).bg(bg),
                    ),
                ]));
            }
        }

        input_lines.push(Line::from(vec![
            Span::styled("  Ref System: ", Style::default().fg(Color::Cyan)),
            Span::styled(self.system_input.clone(), system_style),
            Span::styled(cursor_s, Style::default().fg(Color::Yellow)),
        ]));
        input_lines.push(Line::from(""));
        input_lines.push(Line::from(Span::styled(
            status_text,
            Style::default().fg(Color::DarkGray),
        )));

        let title = if self.editing {
            " Search Nearest  (\u{2193}/\u{2191}: suggestions  |  Tab: complete  |  Enter: search  |  Esc: cancel) "
        } else {
            " Search Nearest  (Enter: edit fields  |  w/s: navigate results  |  c: copy system) "
        };

        frame.render_widget(
            Paragraph::new(input_lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0))),
            ),
            chunks[0],
        );

        // ── Results panel ────────────────────────────────────────
        let mut result_lines: Vec<Line<'static>> = Vec::new();

        if self.results.is_empty() && !self.is_loading {
            result_lines.push(Line::from(Span::styled(
                "  No results.  Enter a commodity and reference system above, then press Enter.",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            result_lines.push(Line::from(Span::styled(
                format!(
                    "  {:>8}  {:<30}  {:<28}  {:>6}  {:>6}  {:>8}  Pad",
                    "Dist ly", "System", "Station", "Buy", "Sell", "Stock"
                ),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            result_lines.push(Line::from(Span::styled(
                "  ".to_string() + &"─".repeat(104),
                Style::default().fg(Color::DarkGray),
            )));

            for (i, r) in self.results.iter().enumerate() {
                let selected = i == self.selected_idx;
                let pad_str = match (r.has_large_pad, r.has_medium_pad) {
                    (true, _) => "L",
                    (false, true) => "M",
                    _ => "S",
                };
                let station_col: String = if r.station_name.chars().count() > 28 {
                    r.station_name.chars().take(27).collect::<String>() + "…"
                } else {
                    format!("{:<28}", r.station_name)
                };
                let system_col: String = if r.system_name.chars().count() > 30 {
                    r.system_name.chars().take(29).collect::<String>() + "…"
                } else {
                    format!("{:<30}", r.system_name)
                };
                let buy_str = if r.buy_price > 0 {
                    r.buy_price.to_string()
                } else {
                    "-".to_string()
                };
                let row = format!(
                    "  {:>8.2}  {}  {}  {:>6}  {:>6}  {:>8}  {}",
                    r.distance_ly,
                    system_col,
                    station_col,
                    buy_str,
                    r.sell_price,
                    r.stock,
                    pad_str,
                );
                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Rgb(255, 140, 0))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                result_lines.push(Line::from(Span::styled(row, style)));
            }
        }

        let visible = chunks[1].height.saturating_sub(2) as usize;
        let scroll = if visible == 0 {
            0
        } else if self.selected_idx + 3 > self.scroll + visible {
            (self.selected_idx + 3).saturating_sub(visible)
        } else if self.selected_idx < self.scroll {
            self.selected_idx
        } else {
            self.scroll
        };
        let max_scroll = result_lines.len().saturating_sub(visible);

        frame.render_widget(
            Paragraph::new(result_lines)
                .block(
                    Block::default()
                        .title(" Results (10 nearest, no fleet carriers) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White)),
                )
                .scroll((scroll.min(max_scroll) as u16, 0)),
            chunks[1],
        );
    }
}
