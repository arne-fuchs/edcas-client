use crate::api_client::ApiClient;
use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::views::util::{fmt_ts, truncate};
use crate::views::ViewEvent;
use edcas_common::api::{TradeLoopResponse, TradeRouteResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

const TYPE_OPTIONS: &[&str] = &["Routes", "Loops"];
const SUPERPOWER_OPTIONS: &[&str] = &["All", "Federation", "Empire", "Alliance", "Independent"];

#[derive(Clone, Copy, PartialEq)]
enum FocusArea {
    Filter,
    List,
    Detail,
}

#[derive(Clone, Copy, PartialEq)]
enum RouteType {
    Routes,
    Loops,
}

pub struct TradeRoutesView {
    focus: FocusArea,
    filter_field: usize,
    type_idx: usize,
    superpower_idx: usize,
    min_supply: i32,
    min_demand: i32,

    routes: Vec<TradeRouteResponse>,
    loops: Vec<TradeLoopResponse>,
    selected_idx: usize,
    scroll: usize,
    detail_cursor: usize,
    status_msg: String,

    #[cfg(not(target_arch = "wasm32"))]
    route_rx: Option<std::sync::mpsc::Receiver<Result<Vec<TradeRouteResponse>, String>>>,
    #[cfg(not(target_arch = "wasm32"))]
    loop_rx: Option<std::sync::mpsc::Receiver<Result<Vec<TradeLoopResponse>, String>>>,
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<arboard::Clipboard>,

    #[cfg(target_arch = "wasm32")]
    pending_routes: Rc<RefCell<Option<Vec<TradeRouteResponse>>>>,
    #[cfg(target_arch = "wasm32")]
    pending_loops: Rc<RefCell<Option<Vec<TradeLoopResponse>>>>,
}

impl TradeRoutesView {
    pub fn new() -> Self {
        Self {
            focus: FocusArea::Filter,
            filter_field: 0,
            type_idx: 0,
            superpower_idx: 0,
            min_supply: 10_000,
            min_demand: 0,
            routes: Vec::new(),
            loops: Vec::new(),
            selected_idx: 0,
            scroll: 0,
            detail_cursor: 0,
            status_msg: "Loading pre-computed trade data…".into(),
            #[cfg(not(target_arch = "wasm32"))]
            route_rx: None,
            #[cfg(not(target_arch = "wasm32"))]
            loop_rx: None,
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: arboard::Clipboard::new().ok(),
            #[cfg(target_arch = "wasm32")]
            pending_routes: Rc::new(RefCell::new(None)),
            #[cfg(target_arch = "wasm32")]
            pending_loops: Rc::new(RefCell::new(None)),
        }
    }

    fn route_type(&self) -> RouteType {
        if self.type_idx == 1 { RouteType::Loops } else { RouteType::Routes }
    }

    pub fn on_enter(&mut self, api: &ApiClient, _journal: &JournalData) {
        if self.routes.is_empty() && self.loops.is_empty() {
            self.fetch_all(api);
        }
    }

    fn fetch_all(&mut self, api: &ApiClient) {
        self.fetch_routes(api);
        self.fetch_loops(api);
    }

    fn passes_route_filter(&self, r: &TradeRouteResponse) -> bool {
        if r.supply < self.min_supply { return false; }
        if r.demand < self.min_demand { return false; }
        if self.superpower_idx == 0 { return true; }
        let sp = SUPERPOWER_OPTIONS[self.superpower_idx];
        let from_ok = r.from_allegiance.is_none() || r.from_allegiance.as_deref() == Some(sp);
        let to_ok   = r.to_allegiance.is_none()   || r.to_allegiance.as_deref()   == Some(sp);
        from_ok && to_ok
    }

    fn passes_loop_filter(&self, l: &TradeLoopResponse) -> bool {
        if l.supply_out < self.min_supply || l.supply_back < self.min_supply { return false; }
        if l.demand_out < self.min_demand || l.demand_back < self.min_demand { return false; }
        if self.superpower_idx == 0 { return true; }
        let sp = SUPERPOWER_OPTIONS[self.superpower_idx];
        let a_ok = l.allegiance_a.is_none() || l.allegiance_a.as_deref() == Some(sp);
        let b_ok = l.allegiance_b.is_none() || l.allegiance_b.as_deref() == Some(sp);
        a_ok && b_ok
    }

    fn filtered_routes(&self) -> Vec<&TradeRouteResponse> {
        self.routes.iter().filter(|r| self.passes_route_filter(r)).collect()
    }

    fn filtered_loops(&self) -> Vec<&TradeLoopResponse> {
        self.loops.iter().filter(|l| self.passes_loop_filter(l)).collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_routes(&mut self, api: &ApiClient) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.route_rx = Some(rx);
        let api_owned = api.clone();
        api.spawn(async move {
            let _ = tx.send(api_owned.fetch_trade_routes().await.map_err(|e| e.to_string()));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_loops(&mut self, api: &ApiClient) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.loop_rx = Some(rx);
        let api_owned = api.clone();
        api.spawn(async move {
            let _ = tx.send(api_owned.fetch_trade_loops().await.map_err(|e| e.to_string()));
        });
        self.status_msg = "Loading…".into();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_results(&mut self) {
        if let Some(rx) = &self.route_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.route_rx = None;
                match outcome {
                    Ok(r) => {
                        self.routes = r;
                        self.refresh_status();
                    }
                    Err(e) => self.status_msg = format!("Routes error: {e}"),
                }
            }
        }
        if let Some(rx) = &self.loop_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.loop_rx = None;
                match outcome {
                    Ok(l) => {
                        self.loops = l;
                        self.refresh_status();
                    }
                    Err(e) => self.status_msg = format!("Loops error: {e}"),
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn fetch_routes(&mut self, api: &ApiClient) {
        let pending = self.pending_routes.clone();
        let client = api.clone();
        wasm_bindgen_futures::spawn_local(async move {
            *pending.borrow_mut() = Some(client.fetch_trade_routes().await);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn fetch_loops(&mut self, api: &ApiClient) {
        let pending = self.pending_loops.clone();
        let client = api.clone();
        wasm_bindgen_futures::spawn_local(async move {
            *pending.borrow_mut() = Some(client.fetch_trade_loops().await);
        });
        self.status_msg = "Loading…".into();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        let routes = self.pending_routes.borrow_mut().take();
        if let Some(r) = routes {
            self.routes = r;
            self.refresh_status();
        }
        let loops = self.pending_loops.borrow_mut().take();
        if let Some(l) = loops {
            self.loops = l;
            self.refresh_status();
        }
    }

    fn refresh_status(&mut self) {
        let r = self.routes.len();
        let l = self.loops.len();
        if r > 0 || l > 0 {
            self.status_msg = format!(
                "{r} routes  {l} loops  |  w/s: navigate  |  Tab: filters  |  c: copy systems  |  r: refresh"
            );
            self.focus = FocusArea::List;
        } else {
            self.status_msg = "No data yet — cache may still be computing".into();
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        match self.focus {
            FocusArea::Filter => self.handle_filter_key(key, api, journal),
            FocusArea::List   => self.handle_list_key(key, api, journal),
            FocusArea::Detail => self.handle_detail_key(key),
        }
    }

    fn handle_filter_key(&mut self, key: &KeyEvent, _api: &ApiClient, _journal: &JournalData) -> ViewEvent {
        match key.code {
            KeyCode::Tab => {
                self.focus = FocusArea::List;
                return ViewEvent::Consumed;
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.filter_field < 3 {
                    self.filter_field += 1;
                } else {
                    self.focus = FocusArea::List;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => {
                if self.filter_field > 0 {
                    self.filter_field -= 1;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Right | KeyCode::Left => {
                let forward = key.code == KeyCode::Right;
                match self.filter_field {
                    0 => {
                        self.type_idx = cycle(self.type_idx, TYPE_OPTIONS.len(), forward);
                        self.selected_idx = 0;
                        self.scroll = 0;
                    }
                    1 => {
                        self.superpower_idx = cycle(self.superpower_idx, SUPERPOWER_OPTIONS.len(), forward);
                        self.selected_idx = 0;
                        self.scroll = 0;
                    }
                    2 => {
                        self.min_supply = if forward {
                            self.min_supply + 1_000
                        } else {
                            (self.min_supply - 1_000).max(0)
                        };
                        self.selected_idx = 0;
                        self.scroll = 0;
                    }
                    3 => {
                        self.min_demand = if forward {
                            self.min_demand + 1_000
                        } else {
                            (self.min_demand - 1_000).max(0)
                        };
                        self.selected_idx = 0;
                        self.scroll = 0;
                    }
                    _ => {}
                }
                return ViewEvent::Consumed;
            }
            _ => {}
        }
        ViewEvent::None
    }

    fn handle_list_key(&mut self, key: &KeyEvent, api: &ApiClient, _journal: &JournalData) -> ViewEvent {
        let count = self.current_count();
        match key.code {
            KeyCode::Tab => {
                self.focus = FocusArea::Detail;
                return ViewEvent::Consumed;
            }
            KeyCode::Up | KeyCode::Char('w') => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                } else {
                    self.focus = FocusArea::Filter;
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if count > 0 {
                    self.selected_idx = (self.selected_idx + 1).min(count - 1);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::PageUp => {
                self.selected_idx = self.selected_idx.saturating_sub(10);
                return ViewEvent::Consumed;
            }
            KeyCode::PageDown => {
                if count > 0 {
                    self.selected_idx = (self.selected_idx + 10).min(count - 1);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('r') => {
                self.fetch_all(api);
                return ViewEvent::Consumed;
            }
            #[cfg(not(target_arch = "wasm32"))]
            KeyCode::Char('c') => {
                let text = match self.route_type() {
                    RouteType::Routes => self
                        .filtered_routes()
                        .into_iter()
                        .nth(self.selected_idx)
                        .map(|r| format!("{} → {}", r.from_system_name, r.to_system_name)),
                    RouteType::Loops => self
                        .filtered_loops()
                        .into_iter()
                        .nth(self.selected_idx)
                        .map(|l| format!("{} ⇄ {}", l.system_name_a, l.system_name_b)),
                };
                if let Some(t) = text {
                    if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(&t);
                        self.status_msg = format!("Copied: {t}");
                    }
                }
                return ViewEvent::Consumed;
            }
            _ => {}
        }
        ViewEvent::None
    }

    fn handle_detail_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Tab => {
                self.focus = FocusArea::Filter;
                ViewEvent::Consumed
            }
            KeyCode::Up | KeyCode::Char('w') => {
                if self.detail_cursor > 0 { self.detail_cursor -= 1; }
                ViewEvent::Consumed
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.detail_cursor = (self.detail_cursor + 1).min(1);
                ViewEvent::Consumed
            }
            #[cfg(not(target_arch = "wasm32"))]
            KeyCode::Char('c') => {
                let text = match self.route_type() {
                    RouteType::Routes => self
                        .filtered_routes()
                        .into_iter()
                        .nth(self.selected_idx)
                        .map(|r| if self.detail_cursor == 0 {
                            r.from_system_name.clone()
                        } else {
                            r.to_system_name.clone()
                        }),
                    RouteType::Loops => self
                        .filtered_loops()
                        .into_iter()
                        .nth(self.selected_idx)
                        .map(|l| if self.detail_cursor == 0 {
                            l.system_name_a.clone()
                        } else {
                            l.system_name_b.clone()
                        }),
                };
                if let Some(t) = text {
                    if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(&t);
                        self.status_msg = format!("Copied: {t}");
                    }
                }
                ViewEvent::Consumed
            }
            _ => ViewEvent::None,
        }
    }

    fn current_count(&self) -> usize {
        match self.route_type() {
            RouteType::Routes => self.filtered_routes().len(),
            RouteType::Loops => self.filtered_loops().len(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, _journal: &JournalData) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        self.render_filter(frame, outer[0]);

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
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
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        };

        let line = Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("[{}]", TYPE_OPTIONS[self.type_idx]), field_style(0)),
            Span::raw("   "),
            Span::styled("Superpower: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("[{}]", SUPERPOWER_OPTIONS[self.superpower_idx]), field_style(1)),
            Span::raw("   "),
            Span::styled("Min Supply: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("[{}]", format_num(self.min_supply)), field_style(2)),
            Span::raw("   "),
            Span::styled("Min Demand: ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("[{}]", format_num(self.min_demand)), field_style(3)),
            Span::styled(
                "   Pre-computed · 500 Ly · Large pad · 15 min refresh",
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        frame.render_widget(
            Paragraph::new(line).block(
                Block::default()
                    .title(" Filters (Tab: next  ←/→: change  r: refresh) ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let visible = area.height.saturating_sub(2) as usize;
        let count = self.current_count();

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
                if self.routes.is_empty() && self.loops.is_empty() {
                    " No data yet — cache may still be computing"
                } else {
                    " No results for selected superpower"
                },
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            match self.route_type() {
                RouteType::Routes => {
                    for (i, r) in self.filtered_routes().into_iter().enumerate().skip(self.scroll).take(visible) {
                        let selected = i == self.selected_idx;
                        let commodity = format!("{:<16}", truncate(&r.commodity, 16));
                        let from = format!("{:<18}", truncate(&r.from_station_name, 18));
                        let to = format!("{:<18}", truncate(&r.to_station_name, 18));
                        let dist = format!("{:>6.1}Ly", r.distance_ly);
                        let profit = format!("{:>8}cr", format_num(r.profit));
                        let style = row_style(selected);
                        lines.push(Line::from(Span::styled(
                            format!(" {commodity} {from} → {to} {dist} {profit}"),
                            style,
                        )));
                    }
                }
                RouteType::Loops => {
                    for (i, l) in self.filtered_loops().into_iter().enumerate().skip(self.scroll).take(visible) {
                        let selected = i == self.selected_idx;
                        let out = format!("{:<14}", truncate(&l.commodity_out, 14));
                        let back = format!("{:<14}", truncate(&l.commodity_back, 14));
                        let sta_a = format!("{:<14}", truncate(&l.station_name_a, 14));
                        let sta_b = format!("{:<14}", truncate(&l.station_name_b, 14));
                        let dist = format!("{:>6.1}Ly", l.distance_ly);
                        let profit = format!("{:>8}cr", format_num(l.total_profit));
                        let style = row_style(selected);
                        lines.push(Line::from(Span::styled(
                            format!(" {out}⇄{back} {sta_a}⇄{sta_b} {dist} {profit}"),
                            style,
                        )));
                    }
                }
            }
        }

        let title = match self.route_type() {
            RouteType::Routes => format!(" Routes ({count})  |  {}", self.status_msg),
            RouteType::Loops => format!(" Loops ({count})  |  {}", self.status_msg),
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let hl = Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
        let focused = self.focus == FocusArea::Detail;
        let border_style = if focused {
            Style::default().fg(Color::Rgb(255, 140, 0))
        } else {
            Style::default().fg(Color::White)
        };
        let sel_style = |pos: usize| -> Style {
            if focused && self.detail_cursor == pos {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        };

        let mut lines: Vec<Line<'static>> = Vec::new();

        match self.route_type() {
            RouteType::Routes => {
                if let Some(r) = self.filtered_routes().into_iter().nth(self.selected_idx) {
                    lines.push(Line::from(Span::styled(r.commodity.clone(), hl)));
                    lines.push(Line::from(""));

                    lines.push(Line::from(Span::styled("Buy from", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {} ({})", r.from_system_name, pad_str(&r.from_max_pad)),
                        sel_style(0),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {}", r.from_station_name),
                        Style::default().fg(Color::Gray),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} cr  ·  supply: {}", format_num(r.buy_price), format_num(r.supply)),
                        Style::default().fg(Color::Green),
                    )));
                    lines.push(Line::from(""));

                    lines.push(Line::from(Span::styled("Sell at", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {} ({})", r.to_system_name, pad_str(&r.to_max_pad)),
                        sel_style(1),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {}", r.to_station_name),
                        Style::default().fg(Color::Gray),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} cr  ·  demand: {}", format_num(r.sell_price), format_num(r.demand)),
                        Style::default().fg(Color::Yellow),
                    )));
                    lines.push(Line::from(""));

                    lines.push(Line::from(Span::styled("Profit / unit", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(format!("  {} cr", format_num(r.profit)), hl)));
                    lines.push(Line::from(""));

                    lines.push(Line::from(Span::styled("Distance", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {:.2} Ly", r.distance_ly),
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("Cache: {}", fmt_ts(r.cached_at.as_ref())),
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    lines.push(no_selection());
                }
            }
            RouteType::Loops => {
                if let Some(l) = self.filtered_loops().into_iter().nth(self.selected_idx) {
                    lines.push(Line::from(Span::styled(
                        format!("Loop  {} cr total", format_num(l.total_profit)),
                        hl,
                    )));
                    lines.push(Line::from(""));

                    // Outbound leg
                    lines.push(Line::from(Span::styled("Outbound →", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {} → {}", l.system_name_a, l.system_name_b),
                        sel_style(0),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} → {}", l.station_name_a, l.station_name_b),
                        Style::default().fg(Color::Gray),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} ({} → {} cr)", l.commodity_out, format_num(l.buy_price_out), format_num(l.sell_price_out)),
                        Style::default().fg(Color::Green),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  profit: {} cr", format_num(l.profit_out)),
                        Style::default().fg(Color::Rgb(255, 140, 0)),
                    )));
                    lines.push(Line::from(""));

                    // Return leg
                    lines.push(Line::from(Span::styled("Return ←", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {} → {}", l.system_name_b, l.system_name_a),
                        sel_style(1),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} → {}", l.station_name_b, l.station_name_a),
                        Style::default().fg(Color::Gray),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  {} ({} → {} cr)", l.commodity_back, format_num(l.buy_price_back), format_num(l.sell_price_back)),
                        Style::default().fg(Color::Yellow),
                    )));
                    lines.push(Line::from(Span::styled(
                        format!("  profit: {} cr", format_num(l.profit_back)),
                        Style::default().fg(Color::Rgb(255, 140, 0)),
                    )));
                    lines.push(Line::from(""));

                    lines.push(Line::from(Span::styled("Distance", Style::default().fg(Color::Cyan))));
                    lines.push(Line::from(Span::styled(
                        format!("  {:.2} Ly  ({})", l.distance_ly, pad_str(&l.max_pad)),
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        format!("Cache: {}", fmt_ts(l.cached_at.as_ref())),
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    lines.push(no_selection());
                }
            }
        }

        let title = if focused {
            " Detail (w/s: select  c: copy system) "
        } else {
            " Detail "
        };

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            ),
            area,
        );
    }
}

fn row_style(selected: bool) -> Style {
    if selected {
        Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

fn no_selection() -> Line<'static> {
    Line::from(Span::styled("No item selected", Style::default().fg(Color::DarkGray)))
}

fn cycle(idx: usize, len: usize, forward: bool) -> usize {
    if forward { (idx + 1) % len } else { (idx + len - 1) % len }
}

fn format_num(n: i32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
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
