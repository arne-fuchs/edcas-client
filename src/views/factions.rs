use crossterm::event::{KeyCode, KeyEvent};
use edcas_common::api::{FactionQuery, FactionResponse};
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
    States,
}

impl DetailTab {
    fn next(self) -> Option<Self> {
        match self {
            Self::Overview => Some(Self::States),
            Self::States => None,
        }
    }
    fn prev(self) -> Option<Self> {
        match self {
            Self::Overview => None,
            Self::States => Some(Self::Overview),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::States => "States",
        }
    }
}

pub struct FactionsView {
    search_query: String,
    search_state: SearchState,
    results: Vec<FactionResponse>,
    selected_idx: usize,
    list_scroll: usize,
    detail_scroll: usize,
    status_msg: String,
    focus: FocusArea,
    detail_tab: DetailTab,
}

impl FactionsView {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            selected_idx: 0,
            list_scroll: 0,
            detail_scroll: 0,
            status_msg: "Press Enter to search".into(),
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
        let query = FactionQuery {
            name: Some(self.search_query.clone()),
            system_name: None,
            limit: Some(200),
        };
        match api.search_factions(&query) {
            Ok(results) => {
                let count = results.len();
                self.results = results;
                self.selected_idx = 0;
                self.list_scroll = 0;
                self.detail_scroll = 0;
                self.focus = FocusArea::List;
                self.detail_tab = DetailTab::Overview;
                self.status_msg = if count == 0 {
                    format!("No factions found for '{}'", self.search_query)
                } else {
                    format!("{count} presence(s) found")
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

        for (i, faction) in self.results.iter().enumerate() {
            let selected = i == self.selected_idx;
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let influence_str = faction
                .influence
                .map(|inf| format!(" ({:.1}%)", inf * 100.0))
                .unwrap_or_default();
            lines.push(Line::from(Span::styled(
                format!(
                    " {}{} — {}",
                    truncate(&faction.name, 26),
                    influence_str,
                    truncate(&faction.system_name, 20),
                ),
                style,
            )));
        }

        lines
    }

    fn build_detail_header_lines(&self) -> Vec<Line<'static>> {
        let tab_active = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [DetailTab::Overview, DetailTab::States];
        let tab_spans: Vec<Span> = tabs
            .iter()
            .flat_map(|&t| {
                let style = if t == self.detail_tab { tab_active } else { tab_inactive };
                [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
            })
            .collect();

        let mut lines = vec![Line::from(tab_spans)];

        if let Some(faction) = self.results.get(self.selected_idx) {
            lines.push(Line::from(Span::styled(
                format!("── {} — {} ──", faction.name, faction.system_name),
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            )));
        }

        lines
    }

    fn build_detail_body_lines(&self) -> Vec<Line<'static>> {
        let Some(faction) = self.results.get(self.selected_idx) else {
            return vec![Line::from(Span::styled(
                "Select a faction from the list.",
                Style::default().fg(Color::DarkGray),
            ))];
        };

        match self.detail_tab {
            DetailTab::Overview => self.overview_body(faction),
            DetailTab::States => self.states_body(faction),
        }
    }

    fn overview_body(&self, faction: &FactionResponse) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!(
            "System:      {}",
            faction.system_name
        )));
        if let Some(ref gov) = faction.government {
            lines.push(Line::from(format!("Government:  {gov}")));
        }
        if let Some(ref alleg) = faction.allegiance {
            lines.push(Line::from(format!("Allegiance:  {alleg}")));
        }
        if let Some(ref hap) = faction.happiness {
            lines.push(Line::from(format!("Happiness:   {hap}")));
        }
        if let Some(inf) = faction.influence {
            let pct = inf * 100.0;
            let filled = (pct / 100.0 * 30.0).round() as usize;
            let bar_color = if pct < 15.0 {
                Color::Red
            } else if pct < 40.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            lines.push(Line::from(vec![
                Span::raw(format!("Influence:   {:>5.1}%  [", pct)),
                Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
                Span::styled("░".repeat(30 - filled), Style::default().fg(Color::DarkGray)),
                Span::raw("]"),
            ]));
        }
        lines
    }

    fn states_body(&self, faction: &FactionResponse) -> Vec<Line<'static>> {
        if faction.states.is_empty() {
            return vec![Line::from(Span::styled(
                "No active states.",
                Style::default().fg(Color::DarkGray),
            ))];
        }
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("{:<32} {}", "State", "Status"),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "─".repeat(44),
            Style::default().fg(Color::DarkGray),
        )));
        for s in &faction.states {
            lines.push(Line::from(format!("{:<32} {}", s.state, s.status)));
        }
        lines
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
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
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
                        .title(" Factions ")
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
            .title(" Faction Details — Enter to search ")
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
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
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
