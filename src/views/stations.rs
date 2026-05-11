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

pub struct StationsView {
    search_query: String,
    search_state: SearchState,
    results: Vec<StationResponse>,
    selected_idx: usize,
    scroll_offset: usize,
    status_msg: String,
}

impl StationsView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            status_msg: "Press '/' to search stations by name or system".into(),
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
                self.scroll_offset = 0;
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

    fn build_lines(&self) -> Vec<Line<'static>> {
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
            lines.push(Line::from("No results. Press '/' to start a search."));
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
                    " {:<40} {:>20}  {}",
                    truncate(&station.name, 40),
                    truncate(&station.system_name, 20),
                    station.station_type.as_deref().unwrap_or(""),
                ),
                style,
            )));
        }

        // Detail panel for selected station
        if let Some(station) = self.results.get(self.selected_idx) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("── {} ──", station.name),
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("  System:    {}", station.system_name)));
            lines.push(Line::from(format!(
                "  Type:      {}",
                station.station_type.as_deref().unwrap_or("Unknown")
            )));
            lines.push(Line::from(format!(
                "  Market ID: {}",
                station.market_id
            )));
            if let Some(ref faction) = station.faction_name {
                lines.push(Line::from(format!("  Faction:   {faction}")));
            }
            if let Some(ref gov) = station.government {
                lines.push(Line::from(format!("  Government:{gov}")));
            }
            if let Some(ref econ) = station.economy {
                lines.push(Line::from(format!("  Economy:   {econ}")));
            }
            if let Some(ref pads) = station.landing_pads {
                lines.push(Line::from(format!(
                    "  Pads:      S:{} M:{} L:{}",
                    pads.small, pads.medium, pads.large
                )));
            }
            if !station.services.is_empty() {
                lines.push(Line::from("  Services:"));
                for chunk in station.services.chunks(4) {
                    lines.push(Line::from(format!("    {}", chunk.join(", "))));
                }
            }
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient) -> ViewEvent {
        match &self.search_state {
            SearchState::Typing => match key.code {
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
            },
            SearchState::Idle => match key.code {
                KeyCode::Char('/') => {
                    self.search_query.clear();
                    self.search_state = SearchState::Typing;
                    self.status_msg = "Typing… (Enter to search, Esc to cancel)".into();
                }
                KeyCode::Char('w') | KeyCode::Up => {
                    self.selected_idx = self.selected_idx.saturating_sub(1);
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    let max = self.results.len().saturating_sub(1);
                    if self.selected_idx < max {
                        self.selected_idx += 1;
                    }
                }
                _ => {}
            },
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self.build_lines();
        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;
        let max_scroll = content_height.saturating_sub(visible_height);

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Stations — / to search ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .scroll((self.scroll_offset.min(max_scroll) as u16, 0));

        frame.render_widget(paragraph, area);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{s:<width$}", width = max)
    } else {
        format!("{:.width$}…", s, width = max - 1)
    }
}
