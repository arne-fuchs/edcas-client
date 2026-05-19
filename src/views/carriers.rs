use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{JournalData, StationData};
use edcas_common::api::{CarrierQuery, CarrierResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashSet;

use crate::api_client::ApiClient;
use crate::views::util::{truncate, FocusArea, SearchState, StationDetailTab as DetailTab};
use crate::views::ViewEvent;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

enum ListItem<'a> {
    Api(&'a CarrierResponse),
    Journal(&'a StationData),
}

pub struct CarriersView {
    search_query: String,
    search_state: SearchState,
    results: Vec<CarrierResponse>,
    pinned_ids: HashSet<i64>,
    pinned_results: Vec<CarrierResponse>,
    selected_idx: usize,
    list_scroll: usize,
    detail_scroll: usize,
    status_msg: String,
    focus: FocusArea,
    detail_tab: DetailTab,
    #[cfg(not(target_arch = "wasm32"))]
    loading_pins: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_pins: Arc<Mutex<Option<Result<Vec<CarrierResponse>, String>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pending_search: Arc<Mutex<Option<Result<Vec<CarrierResponse>, String>>>>,
    #[cfg(target_arch = "wasm32")]
    pending_search: Rc<RefCell<Option<Vec<CarrierResponse>>>>,
}

impl CarriersView {
    pub fn new() -> Self {
        let pins = crate::pins::Pins::load();
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            pinned_ids: pins.carriers,
            pinned_results: Vec::new(),
            selected_idx: 0,
            list_scroll: 0,
            detail_scroll: 0,
            status_msg: "/ or f: search  |  p: pin/unpin".into(),
            focus: FocusArea::List,
            detail_tab: DetailTab::Overview,
            #[cfg(not(target_arch = "wasm32"))]
            loading_pins: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_pins: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            pending_search: Arc::new(Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            pending_search: Rc::new(RefCell::new(None)),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending_search.borrow_mut().take() {
            let count = results.len();
            self.results = results;
            self.selected_idx = 0;
            self.list_scroll = 0;
            self.detail_scroll = 0;
            self.focus = FocusArea::List;
            self.detail_tab = DetailTab::Overview;
            self.status_msg = if count == 0 {
                format!("No carriers found for '{}'", self.search_query)
            } else {
                format!("{count} carrier(s) found")
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_search(&mut self) {
        if let Some(result) = self.pending_pins.lock().unwrap().take() {
            self.loading_pins = false;
            match result {
                Ok(results) => {
                    self.pinned_results = results;
                    if self.status_msg.starts_with("Loading") {
                        self.status_msg = "/ or f: search  |  p: pin/unpin".into();
                    }
                }
                Err(e) => { self.status_msg = format!("API error loading pins: {e}"); }
            }
        }
        if let Some(result) = self.pending_search.lock().unwrap().take() {
            match result {
                Ok(results) => {
                    let count = results.len();
                    self.results = results;
                    self.selected_idx = 0;
                    self.list_scroll = 0;
                    self.detail_scroll = 0;
                    self.focus = FocusArea::List;
                    self.detail_tab = DetailTab::Overview;
                    self.status_msg = if count == 0 {
                        format!("No carriers found for '{}'", self.search_query)
                    } else {
                        format!("{count} carrier(s) found")
                    };
                }
                Err(e) => { self.status_msg = format!("API error: {e}"); }
            }
        }
    }

    pub fn on_enter(&mut self, api: &ApiClient) {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.pinned_ids.is_empty() && self.pinned_results.is_empty() && !self.loading_pins {
            self.refresh_pins(api);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_pins(&mut self, api: &ApiClient) {
        self.loading_pins = true;
        self.pinned_results.clear();
        self.status_msg = "Loading pinned carriers\u{2026}".into();
        let pending = Arc::clone(&self.pending_pins);
        let api_owned = api.clone();
        let ids: Vec<i64> = self.pinned_ids.iter().copied().collect();
        api.spawn(async move {
            let mut results = Vec::new();
            for mid in ids {
                let query = CarrierQuery { market_id: Some(mid), limit: Some(1), ..Default::default() };
                if let Ok(mut r) = api_owned.search_carriers(&query).await {
                    if let Some(c) = r.pop() {
                        results.push(c);
                    }
                }
            }
            results.sort_by(|a, b| a.name.cmp(&b.name));
            *pending.lock().unwrap() = Some(Ok(results));
        });
    }

    fn save_pins(&self) {
        let mut pins = crate::pins::Pins::load();
        pins.carriers = self.pinned_ids.clone();
        pins.save();
    }

    fn build_display_list<'a>(&'a self, history: &'a [StationData]) -> Vec<ListItem<'a>> {
        let search_ids: HashSet<i64> = self.results.iter()
            .filter(|c| !self.pinned_ids.contains(&c.market_id))
            .map(|c| c.market_id)
            .collect();
        let mut list: Vec<ListItem<'a>> = Vec::new();
        for c in &self.pinned_results {
            list.push(ListItem::Api(c));
        }
        for c in self.results.iter().filter(|c| !self.pinned_ids.contains(&c.market_id)) {
            list.push(ListItem::Api(c));
        }
        for c in history.iter().filter(|c| !self.pinned_ids.contains(&c.market_id) && !search_ids.contains(&c.market_id)) {
            list.push(ListItem::Journal(c));
        }
        list
    }

    fn display_count(&self, history: &[StationData]) -> usize {
        self.build_display_list(history).len()
    }

    fn selected_item_with_history<'a>(&'a self, history: &'a [StationData]) -> Option<ListItem<'a>> {
        self.build_display_list(history).into_iter().nth(self.selected_idx)
    }

    fn toggle_pin(&mut self, api: &ApiClient, history: &[StationData]) {
        let n = self.pinned_results.len();
        if self.selected_idx < n {
            let mid = self.pinned_results[self.selected_idx].market_id;
            self.pinned_ids.remove(&mid);
            self.pinned_results.remove(self.selected_idx);
            let total = self.display_count(history);
            if total > 0 {
                self.selected_idx = self.selected_idx.min(total - 1);
            } else {
                self.selected_idx = 0;
            }
        } else {
            let j = self.selected_idx - n;
            let deduped_search: Vec<&CarrierResponse> = self.results.iter()
                .filter(|c| !self.pinned_ids.contains(&c.market_id))
                .collect();
            if j < deduped_search.len() {
                let carrier = deduped_search[j].clone();
                let mid = carrier.market_id;
                self.pinned_ids.insert(mid);
                self.pinned_results.push(carrier);
                self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
                if let Some(pos) = self.pinned_results.iter().position(|c| c.market_id == mid) {
                    self.selected_idx = pos;
                }
            } else {
                let search_ids: HashSet<i64> = self.results.iter().map(|c| c.market_id).collect();
                let hist_deduped: Vec<&StationData> = history.iter()
                    .filter(|c| !self.pinned_ids.contains(&c.market_id) && !search_ids.contains(&c.market_id))
                    .collect();
                let hi = j - deduped_search.len();
                if let Some(h) = hist_deduped.get(hi) {
                    let mid = h.market_id;
                    self.pinned_ids.insert(mid);
                    #[cfg(not(target_arch = "wasm32"))]
                    self.refresh_pins(api);
                    self.selected_idx = 0;
                } else if !self.pinned_ids.is_empty() {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.refresh_pins(api);
                }
            }
        }
        self.save_pins();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_query.is_empty() {
            self.status_msg = "Enter a search term first".into();
            return;
        }
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let pending = Arc::clone(&self.pending_search);
        let api_owned = api.clone();
        let query = CarrierQuery {
            name: Some(self.search_query.clone()),
            callsign: None,
            system_name: None,
            market_id: None,
            limit: Some(50),
        };
        api.spawn(async move {
            let result = api_owned.search_carriers(&query).await.map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_query.is_empty() {
            self.status_msg = "Enter a search term first".into();
            return;
        }
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let pending = Rc::clone(&self.pending_search);
        let api = api.clone();
        let query = CarrierQuery {
            name: Some(self.search_query.clone()),
            callsign: None,
            system_name: None,
            market_id: None,
            limit: Some(50),
        };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_carriers(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    fn build_list_lines(&self, history: &[StationData]) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(self.search_query.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(
                match self.search_state { SearchState::Typing => "_", SearchState::Idle => "" },
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(Span::styled(self.status_msg.clone(), Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(""));

        let n_pinned = self.pinned_results.len();
        let deduped_search: Vec<&CarrierResponse> = self.results.iter()
            .filter(|c| !self.pinned_ids.contains(&c.market_id))
            .collect();
        let search_ids: HashSet<i64> = self.results.iter().map(|c| c.market_id).collect();
        let hist_deduped: Vec<&StationData> = history.iter()
            .filter(|c| !self.pinned_ids.contains(&c.market_id) && !search_ids.contains(&c.market_id))
            .collect();

        for (i, carrier) in self.pinned_results.iter().enumerate() {
            let selected = self.focus == FocusArea::List && i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };
            lines.push(Line::from(Span::styled(
                format!(" \u{2605} {}", carrier_display(carrier, 34)),
                style,
            )));
        }

        let n_search = deduped_search.len();
        if n_search > 0 && n_pinned > 0 {
            lines.push(Line::from(Span::styled(
                "\u{2500}\u{2500}\u{2500} Search \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(Color::DarkGray),
            )));
        }
        for (j, carrier) in deduped_search.iter().enumerate() {
            let i = n_pinned + j;
            let selected = self.focus == FocusArea::List && i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!("   {}", carrier_display(carrier, 34)),
                style,
            )));
        }

        if !hist_deduped.is_empty() {
            let has_above = n_pinned > 0 || n_search > 0;
            if has_above {
                lines.push(Line::from(Span::styled(
                    "\u{2500}\u{2500}\u{2500} History \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            for (k, carrier) in hist_deduped.iter().enumerate() {
                let i = n_pinned + n_search + k;
                let selected = self.focus == FocusArea::List && i == self.selected_idx;
                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(100, 180, 200))
                };
                lines.push(Line::from(Span::styled(
                    format!(" \u{231a} {}", truncate(&carrier.name, 34)),
                    style,
                )));
            }
        }

        if n_pinned == 0 && n_search == 0 && hist_deduped.is_empty() {
            lines.push(Line::from("No results."));
            lines.push(Line::from("Press Enter to search."));
        }

        lines
    }

    fn visual_row_of_selected(&self, history: &[StationData]) -> usize {
        let header = 3usize;
        let n_pinned = self.pinned_results.len();
        let deduped_search: Vec<&CarrierResponse> = self.results.iter()
            .filter(|c| !self.pinned_ids.contains(&c.market_id))
            .collect();
        let n_search = deduped_search.len();
        let search_ids: HashSet<i64> = self.results.iter().map(|c| c.market_id).collect();
        let n_hist = history.iter()
            .filter(|c| !self.pinned_ids.contains(&c.market_id) && !search_ids.contains(&c.market_id))
            .count();

        let has_search_sep = n_pinned > 0 && n_search > 0;
        let has_hist_sep = n_hist > 0 && (n_pinned > 0 || n_search > 0);

        let idx = self.selected_idx;
        if idx < n_pinned {
            header + idx
        } else if idx < n_pinned + n_search {
            header + idx + if has_search_sep { 1 } else { 0 }
        } else {
            header + idx
                + if has_search_sep { 1 } else { 0 }
                + if has_hist_sep { 1 } else { 0 }
        }
    }

    fn build_detail_header_lines(&self, history: &[StationData]) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let tab_active = Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [DetailTab::Overview, DetailTab::Market, DetailTab::Outfitting, DetailTab::Shipyard];
        let tab_spans: Vec<Span> = tabs.iter().flat_map(|&t| {
            let style = if t == self.detail_tab { tab_active } else { tab_inactive };
            [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
        }).collect();
        lines.push(Line::from(tab_spans));

        match self.selected_item_with_history(history) {
            Some(ListItem::Api(carrier)) => {
                if self.detail_tab == DetailTab::Overview {
                    lines.push(Line::from(Span::styled(
                        format!("\u{2500}\u{2500} {} \u{2500}\u{2500}", carrier_display(carrier, 60)),
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                    )));
                } else if self.detail_tab == DetailTab::Market {
                    lines.push(Line::from(Span::styled(
                        format!("{:<28} {:>8} {:>8} {:>8} {:>8} {:>8}", "Commodity", "Buy", "Sell", "Mean", "Stock", "Demand"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(70), Style::default().fg(Color::DarkGray))));
                } else if self.detail_tab == DetailTab::Outfitting {
                    lines.push(Line::from(Span::styled(
                        format!("{:<38} {:>12}", "Module", "Cost"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(52), Style::default().fg(Color::DarkGray))));
                } else if self.detail_tab == DetailTab::Shipyard {
                    lines.push(Line::from(Span::styled(
                        format!("{:<38} {:>14}", "Ship", "Base Value"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(54), Style::default().fg(Color::DarkGray))));
                }
            }
            Some(ListItem::Journal(carrier)) => {
                lines.push(Line::from(Span::styled(
                    format!("\u{2500}\u{2500} {} \u{2500}\u{2500} (visit snapshot)", carrier.name),
                    Style::default().fg(Color::Rgb(100, 180, 200)).add_modifier(Modifier::BOLD),
                )));
            }
            None => {}
        }

        lines
    }

    fn build_detail_body_lines(&self, history: &[StationData]) -> Vec<Line<'static>> {
        match self.selected_item_with_history(history) {
            None => vec![Line::from(Span::styled("Select a carrier from the list.", Style::default().fg(Color::DarkGray)))],
            Some(ListItem::Api(carrier)) => match self.detail_tab {
                DetailTab::Overview => self.overview_body(carrier),
                DetailTab::Market => self.market_body(carrier),
                DetailTab::Outfitting => self.outfitting_body(carrier),
                DetailTab::Shipyard => self.shipyard_body(carrier),
            },
            Some(ListItem::Journal(carrier)) => match self.detail_tab {
                DetailTab::Overview => self.journal_overview_body(carrier),
                _ => vec![Line::from(Span::styled(
                    "No live data \u{2014} this is a visit snapshot.  Pin (p) to load full data.",
                    Style::default().fg(Color::DarkGray),
                ))],
            },
        }
    }

    fn overview_body(&self, carrier: &CarrierResponse) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("Callsign:  {}", carrier.name)));
        if let Some(ref cn) = carrier.carrier_name {
            lines.push(Line::from(format!("Name:      {cn}")));
        }
        lines.push(Line::from(format!("System:    {}", carrier.system_name)));
        lines.push(Line::from(format!("Market ID: {}", carrier.market_id)));
        if let Some(ref faction) = carrier.faction_name { lines.push(Line::from(format!("Faction:   {faction}"))); }
        if !carrier.services.is_empty() {
            lines.push(Line::from("Services:"));
            for chunk in carrier.services.chunks(3) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
        }
        lines
    }

    fn journal_overview_body(&self, carrier: &StationData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("Callsign:  {}", carrier.name)));
        lines.push(Line::from(format!("System:    {}", carrier.system_name)));
        lines.push(Line::from(format!("Market ID: {}", carrier.market_id)));
        if !carrier.faction.is_empty() { lines.push(Line::from(format!("Faction:   {}", carrier.faction))); }
        if !carrier.services.is_empty() {
            lines.push(Line::from("Services:"));
            for chunk in carrier.services.chunks(3) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Visit snapshot \u{2014} pin (p) to load live market data.",
            Style::default().fg(Color::DarkGray),
        )));
        lines
    }

    fn market_body(&self, carrier: &CarrierResponse) -> Vec<Line<'static>> {
        if carrier.commodities.is_empty() {
            return vec![Line::from(Span::styled("No market data available.", Style::default().fg(Color::DarkGray)))];
        }
        carrier.commodities.iter().map(|c| {
            let buy = if c.buy_price > 0 { format!("{:>8}", c.buy_price) } else { format!("{:>8}", "-") };
            let sell = if c.sell_price > 0 { format!("{:>8}", c.sell_price) } else { format!("{:>8}", "-") };
            Line::from(format!("{:<28} {} {} {:>8} {:>8} {:>8}", truncate(&c.name, 28), buy, sell, c.mean_price, c.stock, c.demand))
        }).collect()
    }

    fn outfitting_body(&self, carrier: &CarrierResponse) -> Vec<Line<'static>> {
        if carrier.modules.is_empty() {
            return vec![Line::from(Span::styled("No outfitting data available.", Style::default().fg(Color::DarkGray)))];
        }
        let mut lines = Vec::new();
        let mut last_cat = String::new();
        for m in &carrier.modules {
            let cat = m.category.as_deref().unwrap_or("");
            if cat != last_cat {
                if !last_cat.is_empty() { lines.push(Line::from("")); }
                lines.push(Line::from(Span::styled(format!("[{cat}]"), Style::default().fg(Color::Yellow))));
                last_cat = cat.to_owned();
            }
            let name = m.name.as_deref().unwrap_or(&m.id);
            let cost = if m.cost > 0 { format!("{:>12}", m.cost) } else { format!("{:>12}", "-") };
            lines.push(Line::from(format!("  {:<36} {}", truncate(name, 36), cost)));
        }
        lines
    }

    fn shipyard_body(&self, carrier: &CarrierResponse) -> Vec<Line<'static>> {
        if carrier.ships.is_empty() {
            return vec![Line::from(Span::styled("No shipyard data available.", Style::default().fg(Color::DarkGray)))];
        }
        carrier.ships.iter().map(|s| {
            let name = s.name.as_deref().unwrap_or(&s.id);
            let val = if s.basevalue > 0 { format!("{:>14}", s.basevalue) } else { format!("{:>14}", "-") };
            Line::from(format!("{:<38} {}", truncate(name, 38), val))
        }).collect()
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        let history = &journal.visited_carriers;
        if matches!(self.search_state, SearchState::Typing) {
            match key.code {
                KeyCode::Esc => { self.search_state = SearchState::Idle; self.status_msg = "Search cancelled".into(); }
                KeyCode::Enter => { self.search_state = SearchState::Idle; self.do_search(api); }
                KeyCode::Backspace => { self.search_query.pop(); }
                KeyCode::Char(c) => { self.search_query.push(c); }
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Char('/') | KeyCode::Char('f') => {
                self.search_query.clear();
                self.search_state = SearchState::Typing;
                self.status_msg = "Typing\u{2026} (Enter to search, Esc to cancel)".into();
            }
            KeyCode::Char('p') => {
                if self.display_count(history) > 0 {
                    self.toggle_pin(api, history);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => match self.focus {
                FocusArea::List => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(1); }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.focus {
                FocusArea::List => {
                    if self.selected_idx + 1 < self.display_count(history) {
                        self.selected_idx += 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => { self.detail_scroll += 1; }
            },
            KeyCode::PageUp => match self.focus {
                FocusArea::List => { self.selected_idx = self.selected_idx.saturating_sub(10); }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(10); }
            },
            KeyCode::PageDown => match self.focus {
                FocusArea::List => {
                    let max = self.display_count(history).saturating_sub(1);
                    self.selected_idx = (self.selected_idx + 10).min(max);
                }
                FocusArea::Detail => { self.detail_scroll += 10; }
            },
            KeyCode::Tab => {
                match self.focus {
                    FocusArea::List => {
                        if self.display_count(history) > 0 {
                            self.focus = FocusArea::Detail;
                            self.detail_scroll = 0;
                        }
                    }
                    FocusArea::Detail => { self.focus = FocusArea::List; }
                }
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if self.focus == FocusArea::Detail {
                    if let Some(next) = self.detail_tab.next() {
                        self.detail_tab = next;
                        self.detail_scroll = 0;
                    }
                }
            }
            KeyCode::Char('a') | KeyCode::Left => {
                if self.focus == FocusArea::Detail {
                    if let Some(prev) = self.detail_tab.prev() {
                        self.detail_tab = prev;
                        self.detail_scroll = 0;
                    }
                }
            }
            _ => {}
        }

        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let history = &journal.visited_carriers;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        let list_lines = self.build_list_lines(history);
        let list_height = chunks[0].height.saturating_sub(2) as usize;
        let row = self.visual_row_of_selected(history);
        let list_scroll = if row + 1 >= self.list_scroll + list_height {
            (row + 2).saturating_sub(list_height)
        } else if row < self.list_scroll {
            row.saturating_sub(1)
        } else {
            self.list_scroll
        };

        frame.render_widget(
            Paragraph::new(list_lines)
                .block(Block::default().title(" Fleet Carriers ").borders(Borders::ALL).border_style(
                    if self.focus == FocusArea::List { active_border } else { inactive_border },
                ))
                .scroll((list_scroll as u16, 0)),
            chunks[0],
        );

        let detail_block = Block::default()
            .title(" Carrier Details \u{2014} Enter to search ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail { active_border } else { inactive_border });
        let detail_inner = detail_block.inner(chunks[1]);
        frame.render_widget(detail_block, chunks[1]);

        let header_lines = self.build_detail_header_lines(history);
        let header_height = header_lines.len() as u16;
        let detail_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .split(detail_inner);

        frame.render_widget(Paragraph::new(header_lines), detail_split[0]);

        let body_lines = self.build_detail_body_lines(history);
        let body_height = detail_split[1].height as usize;
        let body_max_scroll = body_lines.len().saturating_sub(body_height);
        frame.render_widget(
            Paragraph::new(body_lines).scroll((self.detail_scroll.min(body_max_scroll) as u16, 0)),
            detail_split[1],
        );
    }
}

fn carrier_display(carrier: &CarrierResponse, max: usize) -> String {
    let s = match carrier.carrier_name.as_deref() {
        Some(cn) => format!("{cn} ({})", carrier.name),
        None => carrier.name.clone(),
    };
    truncate(&s, max)
}
