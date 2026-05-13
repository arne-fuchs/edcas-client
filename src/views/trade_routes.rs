use crate::api_client::ApiClient;
use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::views::ViewEvent;
use edcas_common::api::{TradeRouteQuery, TradeRouteResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

const PAD_OPTIONS: &[&str] = &["Any", "L", "M+"];

#[derive(Clone, Copy, PartialEq)]
enum FocusArea {
    Filter,
    List,
}

pub struct TradeRoutesView {
    focus: FocusArea,
    filter_field: usize,
    max_distance_input: String,
    pad_idx: usize,
    min_profit_input: String,
    results: Vec<TradeRouteResponse>,
    selected_idx: usize,
    scroll: usize,
    status_msg: String,
    #[cfg(not(target_arch = "wasm32"))]
    search_rx: Option<std::sync::mpsc::Receiver<Result<Vec<TradeRouteResponse>, String>>>,
    #[cfg(target_arch = "wasm32")]
    pending: Rc<RefCell<Option<Vec<TradeRouteResponse>>>>,
}

impl TradeRoutesView {
    pub fn new() -> Self {
        Self {
            focus: FocusArea::Filter,
            filter_field: 0,
            max_distance_input: "200".into(),
            pad_idx: 0,
            min_profit_input: "1000".into(),
            results: Vec::new(),
            selected_idx: 0,
            scroll: 0,
            status_msg: "Enter system to search around, then press Enter on [Search]".into(),
            #[cfg(not(target_arch = "wasm32"))]
            search_rx: None,
            #[cfg(target_arch = "wasm32")]
            pending: Rc::new(RefCell::new(None)),
        }
    }

    pub fn on_enter(&mut self, api: &ApiClient, journal: &JournalData) {
        if self.results.is_empty() {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(sys) = &journal.current_system {
                self.do_search(api, sys.system_address);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending.borrow_mut().take() {
            let n = results.len();
            self.results = results;
            self.selected_idx = 0;
            self.scroll = 0;
            self.focus = FocusArea::List;
            self.status_msg = if n == 0 {
                "No routes found — try wider distance or lower profit threshold".into()
            } else {
                format!("{n} routes found  |  w/s: navigate  |  Tab: filter")
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient, system_address: i64) {
        let max_dist = self.max_distance_input.parse::<f32>().unwrap_or(200.0);
        let min_profit = self.min_profit_input.parse::<i32>().unwrap_or(1000);
        let pad = match self.pad_idx {
            1 => Some("L".to_string()),
            2 => Some("M".to_string()),
            _ => None,
        };
        let query = TradeRouteQuery {
            system_address: Some(system_address),
            max_distance: Some(max_dist),
            pad_size: pad,
            min_profit: Some(min_profit),
            limit: Some(100),
        };
        let base_url = api.base_url().to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_rx = Some(rx);
        std::thread::spawn(move || {
            let client = ApiClient::new(base_url);
            let result = client
                .fetch_trade_routes(&query)
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        self.status_msg = "Searching…".into();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_results(&mut self) {
        let result = match &self.search_rx {
            Some(rx) => rx.try_recv().ok(),
            None => return,
        };
        if let Some(outcome) = result {
            self.search_rx = None;
            match outcome {
                Ok(results) => {
                    let n = results.len();
                    self.results = results;
                    self.selected_idx = 0;
                    self.scroll = 0;
                    self.focus = FocusArea::List;
                    self.status_msg = if n == 0 {
                        "No routes found — try wider distance or lower profit threshold".into()
                    } else {
                        format!("{n} routes found  |  w/s: navigate  |  Tab: filter")
                    };
                }
                Err(e) => {
                    self.status_msg = format!("Search failed: {e}");
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient, system_address: i64) {
        let max_dist = self.max_distance_input.parse::<f32>().unwrap_or(200.0);
        let min_profit = self.min_profit_input.parse::<i32>().unwrap_or(1000);
        let pad = match self.pad_idx {
            1 => Some("L".to_string()),
            2 => Some("M".to_string()),
            _ => None,
        };
        let query = TradeRouteQuery {
            system_address: Some(system_address),
            max_distance: Some(max_dist),
            pad_size: pad,
            min_profit: Some(min_profit),
            limit: Some(100),
        };
        let pending = self.pending.clone();
        let client = api.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let results = client.fetch_trade_routes(query).await;
            *pending.borrow_mut() = Some(results);
        });
        self.status_msg = "Searching…".into();
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        match self.focus {
            FocusArea::Filter => self.handle_filter_key(key, api, journal),
            FocusArea::List => self.handle_list_key(key, api, journal),
        }
    }

    fn handle_filter_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        match key.code {
            KeyCode::Tab => {
                if self.filter_field < 3 {
                    self.filter_field += 1;
                } else {
                    self.focus = FocusArea::List;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::BackTab => {
                if self.filter_field > 0 {
                    self.filter_field -= 1;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Right => {
                if self.filter_field == 1 {
                    self.pad_idx = (self.pad_idx + 1) % PAD_OPTIONS.len();
                } else {
                    self.filter_field = (self.filter_field + 1).min(3);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Left => {
                if self.filter_field == 1 {
                    self.pad_idx = (self.pad_idx + PAD_OPTIONS.len() - 1) % PAD_OPTIONS.len();
                } else if self.filter_field > 0 {
                    self.filter_field -= 1;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Enter => {
                if self.filter_field == 3 {
                    if let Some(sys) = &journal.current_system {
                        self.do_search(api, sys.system_address);
                    } else {
                        self.status_msg = "No current system — load a journal first".into();
                    }
                } else {
                    self.filter_field = (self.filter_field + 1).min(3);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Backspace => {
                match self.filter_field {
                    0 => { self.max_distance_input.pop(); }
                    2 => { self.min_profit_input.pop(); }
                    _ => {}
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char(c) => {
                if c == 's' {
                    self.focus = FocusArea::List;
                    return ViewEvent::Consumed;
                }
                if c.is_ascii_digit() {
                    match self.filter_field {
                        0 => self.max_distance_input.push(c),
                        2 => self.min_profit_input.push(c),
                        _ => {}
                    }
                    return ViewEvent::Consumed;
                }
            }
            KeyCode::Down => {
                self.focus = FocusArea::List;
                return ViewEvent::Consumed;
            }
            _ => {}
        }
        ViewEvent::None
    }

    fn handle_list_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                } else {
                    self.focus = FocusArea::Filter;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if !self.results.is_empty() {
                    self.selected_idx = (self.selected_idx + 1).min(self.results.len() - 1);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::PageUp => {
                self.selected_idx = self.selected_idx.saturating_sub(10);
                return ViewEvent::Consumed;
            }
            KeyCode::PageDown => {
                if !self.results.is_empty() {
                    self.selected_idx = (self.selected_idx + 10).min(self.results.len() - 1);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Tab => {
                self.focus = FocusArea::Filter;
                self.filter_field = 3;
                return ViewEvent::Consumed;
            }
            KeyCode::Char('r') => {
                if let Some(sys) = &journal.current_system {
                    self.do_search(api, sys.system_address);
                }
                return ViewEvent::Consumed;
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, _journal: &JournalData) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.render_filter(frame, outer[0]);

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(outer[1]);

        self.render_list(frame, inner[0]);
        self.render_detail(frame, inner[1]);
    }

    fn render_filter(&self, frame: &mut Frame, area: Rect) {
        let focused = self.focus == FocusArea::Filter;
        let border_style = if focused {
            Style::default().fg(Color::Rgb(255, 140, 0))
        } else {
            Style::default().fg(Color::White)
        };

        let field_style = |idx: usize| -> Style {
            if focused && self.filter_field == idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        };

        let dist_label = Span::styled("Max Dist: ", Style::default().fg(Color::Cyan));
        let dist_val = Span::styled(
            format!("[{} Ly]", self.max_distance_input),
            field_style(0),
        );
        let sep = Span::raw("  ");
        let pad_label = Span::styled("Pad: ", Style::default().fg(Color::Cyan));
        let pad_val = Span::styled(
            format!("[{}]", PAD_OPTIONS[self.pad_idx]),
            field_style(1),
        );
        let profit_label = Span::styled("  Min Profit: ", Style::default().fg(Color::Cyan));
        let profit_val = Span::styled(
            format!("[{} cr]", self.min_profit_input),
            field_style(2),
        );
        let search_btn = Span::styled(
            "  [Search]",
            if focused && self.filter_field == 3 {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(255, 140, 0))
            },
        );

        let line = Line::from(vec![
            dist_label, dist_val, sep, pad_label, pad_val,
            profit_label, profit_val, search_btn,
        ]);

        frame.render_widget(
            Paragraph::new(line).block(
                Block::default()
                    .title(" Filters (Tab: next field  ←/→: change  Enter: search) ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let visible = area.height.saturating_sub(2) as usize;
        let count = self.results.len();

        if self.selected_idx < self.scroll {
            self.scroll = self.selected_idx;
        } else if self.selected_idx >= self.scroll + visible {
            self.scroll = self.selected_idx + 1 - visible;
        }

        let focused = self.focus == FocusArea::List;
        let border_style = if focused {
            Style::default().fg(Color::Rgb(255, 140, 0))
        } else {
            Style::default().fg(Color::White)
        };

        let mut lines: Vec<Line<'static>> = Vec::new();

        if count == 0 {
            lines.push(Line::from(Span::styled(
                " No results yet — configure filters and press [Search]",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (i, route) in self.results.iter().enumerate().skip(self.scroll).take(visible) {
                let selected = i == self.selected_idx;
                let commodity = format!("{:<16}", truncate(&route.commodity, 16));
                let from = format!("{:<20}", truncate(&route.from_station_name, 20));
                let to = format!("{:<20}", truncate(&route.to_station_name, 20));
                let dist = format!("{:>6.1}Ly", route.distance_ly);
                let profit = format!("{:>7}", format_num(route.profit));

                let style = if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Rgb(255, 140, 0))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(Span::styled(
                    format!(" {commodity} {from} → {to} {dist} {profit}cr"),
                    style,
                )));
            }
        }

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(format!(" Trade Routes ({count})  |  {}", self.status_msg))
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if let Some(r) = self.results.get(self.selected_idx) {
            let hl = Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);

            lines.push(Line::from(Span::styled(r.commodity.clone(), hl)));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Buy from", Style::default().fg(Color::Cyan))));
            lines.push(Line::from(Span::styled(r.from_station_name.clone(), Style::default().fg(Color::White))));
            lines.push(Line::from(Span::styled(
                format!("  {} ({})", r.from_system_name, pad_str(&r.from_max_pad)),
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {} cr  ·  supply: {}", format_num(r.buy_price), format_num(r.supply)),
                Style::default().fg(Color::Green),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Sell at", Style::default().fg(Color::Cyan))));
            lines.push(Line::from(Span::styled(r.to_station_name.clone(), Style::default().fg(Color::White))));
            lines.push(Line::from(Span::styled(
                format!("  {} ({})", r.to_system_name, pad_str(&r.to_max_pad)),
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {} cr  ·  demand: {}", format_num(r.sell_price), format_num(r.demand)),
                Style::default().fg(Color::Yellow),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Profit / unit", Style::default().fg(Color::Cyan))));
            lines.push(Line::from(Span::styled(
                format!("  {} cr", format_num(r.profit)),
                hl,
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled("Distance", Style::default().fg(Color::Cyan))));
            lines.push(Line::from(Span::styled(
                format!("  {:.2} Ly", r.distance_ly),
                Style::default().fg(Color::White),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No route selected",
                Style::default().fg(Color::DarkGray),
            )));
        }

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Detail ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            ),
            area,
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

fn format_num(n: i32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn pad_str(pad: &Option<String>) -> &str {
    match pad.as_deref() {
        Some("L") => "Large pad",
        Some("M") => "Medium pad",
        Some("S") => "Small pad",
        _ => "Unknown pad",
    }
}
