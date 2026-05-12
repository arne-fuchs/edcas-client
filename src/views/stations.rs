use crossterm::event::{KeyCode, KeyEvent};
use edcas_common::api::{StationQuery, StationResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::api_client::ApiClient;
use crate::views::ViewEvent;

enum SearchState {
    Idle,
    Typing,
}

#[derive(Clone, Copy, PartialEq)]
enum FocusArea {
    List,
    Detail,
}

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Overview,
    Market,
    Outfitting,
    Shipyard,
}

impl DetailTab {
    fn next(self) -> Option<Self> {
        match self {
            Self::Overview => Some(Self::Market),
            Self::Market => Some(Self::Outfitting),
            Self::Outfitting => Some(Self::Shipyard),
            Self::Shipyard => None,
        }
    }
    fn prev(self) -> Option<Self> {
        match self {
            Self::Overview => None,
            Self::Market => Some(Self::Overview),
            Self::Outfitting => Some(Self::Market),
            Self::Shipyard => Some(Self::Outfitting),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Market => "Market",
            Self::Outfitting => "Outfitting",
            Self::Shipyard => "Shipyard",
        }
    }
}

pub struct StationsView {
    search_query: String,
    search_state: SearchState,
    results: Vec<StationResponse>,
    selected_idx: usize,
    list_scroll: usize,
    detail_scroll: usize,
    status_msg: String,
    focus: FocusArea,
    detail_tab: DetailTab,
}

impl StationsView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            selected_idx: 0,
            list_scroll: 0,
            detail_scroll: 0,
            status_msg: "Press Enter to search, d to open detail".into(),
            focus: FocusArea::List,
            detail_tab: DetailTab::Overview,
        }
    }

    fn do_search(&mut self, api: &ApiClient) {
        if self.search_query.is_empty() {
            self.status_msg = "Enter a search term first".into();
            return;
        }
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let query = StationQuery {
            name: Some(self.search_query.clone()),
            system_name: None,
            market_id: None,
            limit: Some(50),
        };
        match api.search_stations(&query) {
            Ok(results) => {
                let count = results.len();
                self.results = results;
                self.selected_idx = 0;
                self.list_scroll = 0;
                self.detail_scroll = 0;
                self.focus = FocusArea::List;
                self.detail_tab = DetailTab::Overview;
                self.status_msg = if count == 0 {
                    format!("No stations found for '{}'", self.search_query)
                } else {
                    format!("{count} station(s) found")
                };
            }
            Err(e) => {
                self.status_msg = format!("API error: {e}");
            }
        }
    }

    fn build_list_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                self.search_query.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                match self.search_state {
                    SearchState::Typing => "_",
                    SearchState::Idle => "",
                },
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            self.status_msg.clone(),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        if self.results.is_empty() {
            lines.push(Line::from("No results."));
            lines.push(Line::from("Press Enter to search."));
            return lines;
        }

        for (i, station) in self.results.iter().enumerate() {
            let selected = i == self.selected_idx;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!(
                    " {:<28} {}",
                    truncate(&station.name, 28),
                    station.station_type.as_deref().unwrap_or(""),
                ),
                style,
            )));
        }

        lines
    }

    // Fixed header: tab bar + per-tab column titles
    fn build_detail_header_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Tab bar
        let tab_active = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [
            DetailTab::Overview,
            DetailTab::Market,
            DetailTab::Outfitting,
            DetailTab::Shipyard,
        ];
        let tab_spans: Vec<Span> = tabs
            .iter()
            .flat_map(|&t| {
                let style = if t == self.detail_tab { tab_active } else { tab_inactive };
                [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
            })
            .collect();
        lines.push(Line::from(tab_spans));

        let Some(station) = self.results.get(self.selected_idx) else {
            return lines;
        };

        match self.detail_tab {
            DetailTab::Overview => {
                lines.push(Line::from(Span::styled(
                    format!("── {} ──", station.name),
                    Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                )));
            }
            DetailTab::Market => {
                lines.push(Line::from(Span::styled(
                    format!(
                        "{:<28} {:>8} {:>8} {:>8} {:>8} {:>8}",
                        "Commodity", "Buy", "Sell", "Mean", "Stock", "Demand"
                    ),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "─".repeat(70),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            DetailTab::Outfitting => {
                lines.push(Line::from(Span::styled(
                    format!("{:<38} {:>12}", "Module", "Cost"),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "─".repeat(52),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            DetailTab::Shipyard => {
                lines.push(Line::from(Span::styled(
                    format!("{:<38} {:>14}", "Ship", "Base Value"),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "─".repeat(54),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        lines
    }

    // Scrollable body: only the data rows
    fn build_detail_body_lines(&self) -> Vec<Line<'static>> {
        let Some(station) = self.results.get(self.selected_idx) else {
            return vec![Line::from(Span::styled(
                "Select a station from the list.",
                Style::default().fg(Color::DarkGray),
            ))];
        };

        match self.detail_tab {
            DetailTab::Overview => self.overview_body(station),
            DetailTab::Market => self.market_body(station),
            DetailTab::Outfitting => self.outfitting_body(station),
            DetailTab::Shipyard => self.shipyard_body(station),
        }
    }

    fn overview_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("System:    {}", station.system_name)));
        lines.push(Line::from(format!(
            "Type:      {}",
            station.station_type.as_deref().unwrap_or("Unknown")
        )));
        lines.push(Line::from(format!("Market ID: {}", station.market_id)));
        if let Some(ref faction) = station.faction_name {
            lines.push(Line::from(format!("Faction:   {faction}")));
        }
        if let Some(ref gov) = station.government {
            lines.push(Line::from(format!("Government:{gov}")));
        }
        if let Some(ref econ) = station.economy {
            lines.push(Line::from(format!("Economy:   {econ}")));
        }
        if let Some(ref pads) = station.landing_pads {
            lines.push(Line::from(format!(
                "Pads:      S:{} M:{} L:{}",
                pads.small, pads.medium, pads.large
            )));
        }
        if !station.services.is_empty() {
            lines.push(Line::from("Services:"));
            for chunk in station.services.chunks(3) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
        }
        lines
    }

    fn market_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        if station.commodities.is_empty() {
            return vec![Line::from(Span::styled(
                "No market data available.",
                Style::default().fg(Color::DarkGray),
            ))];
        }
        station.commodities.iter().map(|c| {
            let buy = if c.buy_price > 0 { format!("{:>8}", c.buy_price) } else { format!("{:>8}", "-") };
            let sell = if c.sell_price > 0 { format!("{:>8}", c.sell_price) } else { format!("{:>8}", "-") };
            Line::from(format!(
                "{:<28} {} {} {:>8} {:>8} {:>8}",
                truncate(&c.name, 28),
                buy,
                sell,
                c.mean_price,
                c.stock,
                c.demand,
            ))
        }).collect()
    }

    fn outfitting_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        if station.modules.is_empty() {
            return vec![Line::from(Span::styled(
                "No outfitting data available.",
                Style::default().fg(Color::DarkGray),
            ))];
        }
        let mut lines = Vec::new();
        let mut last_cat = String::new();
        for m in &station.modules {
            let cat = m.category.as_deref().unwrap_or("");
            if cat != last_cat {
                if !last_cat.is_empty() {
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    format!("[{cat}]"),
                    Style::default().fg(Color::Yellow),
                )));
                last_cat = cat.to_owned();
            }
            let name = m.name.as_deref().unwrap_or(&m.id);
            let cost = if m.cost > 0 { format!("{:>12}", m.cost) } else { format!("{:>12}", "-") };
            lines.push(Line::from(format!("  {:<36} {}", truncate(name, 36), cost)));
        }
        lines
    }

    fn shipyard_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        if station.ships.is_empty() {
            return vec![Line::from(Span::styled(
                "No shipyard data available.",
                Style::default().fg(Color::DarkGray),
            ))];
        }
        station.ships.iter().map(|s| {
            let name = s.name.as_deref().unwrap_or(&s.id);
            let val = if s.basevalue > 0 { format!("{:>14}", s.basevalue) } else { format!("{:>14}", "-") };
            Line::from(format!("{:<38} {}", truncate(name, 38), val))
        }).collect()
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient) -> ViewEvent {
        if matches!(self.search_state, SearchState::Typing) {
            match key.code {
                KeyCode::Esc => {
                    self.search_state = SearchState::Idle;
                    self.status_msg = "Search cancelled".into();
                }
                KeyCode::Enter => {
                    self.search_state = SearchState::Idle;
                    self.do_search(api);
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Enter => {
                self.search_query.clear();
                self.search_state = SearchState::Typing;
                self.status_msg = "Typing… (Enter to search, Esc to cancel)".into();
            }
            KeyCode::Char('w') | KeyCode::Up => match self.focus {
                FocusArea::List => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.focus {
                FocusArea::List => {
                    if self.selected_idx + 1 < self.results.len() {
                        self.selected_idx += 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => {
                    self.detail_scroll += 1;
                }
            },
            KeyCode::Char('d') | KeyCode::Right => match self.focus {
                FocusArea::List => {
                    if !self.results.is_empty() {
                        self.focus = FocusArea::Detail;
                        self.detail_scroll = 0;
                    }
                }
                FocusArea::Detail => {
                    if let Some(next) = self.detail_tab.next() {
                        self.detail_tab = next;
                        self.detail_scroll = 0;
                    }
                }
            },
            KeyCode::Char('a') | KeyCode::Left => match self.focus {
                FocusArea::List => {}
                FocusArea::Detail => match self.detail_tab.prev() {
                    Some(prev) => {
                        self.detail_tab = prev;
                        self.detail_scroll = 0;
                    }
                    None => {
                        self.focus = FocusArea::List;
                    }
                },
            },
            _ => {}
        }

        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        // ── Left: list ───────────────────────────────────────────
        let list_lines = self.build_list_lines();
        let list_height = chunks[0].height.saturating_sub(2) as usize;
        let list_scroll = if self.selected_idx + 3 >= self.list_scroll + list_height {
            (self.selected_idx + 4).saturating_sub(list_height)
        } else if self.selected_idx + 3 < self.list_scroll {
            self.selected_idx + 3
        } else {
            self.list_scroll
        };

        frame.render_widget(
            Paragraph::new(list_lines)
                .block(
                    Block::default()
                        .title(" Stations ")
                        .borders(Borders::ALL)
                        .border_style(if self.focus == FocusArea::List {
                            active_border
                        } else {
                            inactive_border
                        }),
                )
                .scroll((list_scroll as u16, 0)),
            chunks[0],
        );

        // ── Right: detail — fixed header + scrollable body ───────
        let detail_block = Block::default()
            .title(" Station Details — Enter to search ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail {
                active_border
            } else {
                inactive_border
            });
        let detail_inner = detail_block.inner(chunks[1]);
        frame.render_widget(detail_block, chunks[1]);

        let header_lines = self.build_detail_header_lines();
        let header_height = header_lines.len() as u16;

        let detail_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(0),
            ])
            .split(detail_inner);

        frame.render_widget(Paragraph::new(header_lines), detail_split[0]);

        let body_lines = self.build_detail_body_lines();
        let body_height = detail_split[1].height as usize;
        let body_max_scroll = body_lines.len().saturating_sub(body_height);

        frame.render_widget(
            Paragraph::new(body_lines)
                .scroll((self.detail_scroll.min(body_max_scroll) as u16, 0)),
            detail_split[1],
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{s:<width$}", width = max)
    } else {
        format!("{:.width$}…", s, width = max - 1)
    }
}
