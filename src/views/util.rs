use std::cmp::Ordering;
use std::collections::HashMap;

use edcas_common::api::{CommodityResponse, ModuleResponse, ShipResponse};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub(super) fn normalize_commodity_name(name: &str) -> String {
    name.trim_start_matches('$')
        .trim_end_matches(';')
        .trim_end_matches("_name")
        .to_lowercase()
}

/// Sortable market columns, used by both station and carrier views.
#[derive(Clone, Copy, PartialEq, Default)]
pub(super) enum MarketSortCol {
    #[default]
    Name,
    Buy,
    BuyDiff,
    Sell,
    SellDiff,
    Mean,
    Stock,
    Demand,
}

impl MarketSortCol {
    pub(super) fn from_digit(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Name),
            '2' => Some(Self::Buy),
            '3' => Some(Self::BuyDiff),
            '4' => Some(Self::Sell),
            '5' => Some(Self::SellDiff),
            '6' => Some(Self::Mean),
            '7' => Some(Self::Stock),
            '8' => Some(Self::Demand),
            _ => None,
        }
    }
}

/// Build a dynamic column-header Line with ▲/▼ on the active sort column.
pub(super) fn commodity_header_line(sort_col: MarketSortCol, sort_asc: bool) -> Line<'static> {
    let arrow: &'static str = if sort_asc { "\u{25b2}" } else { "\u{25bc}" };
    let cyan_bold = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let active   = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);

    let span = move |col: MarketSortCol, label: &'static str, width: usize, leading: bool| {
        let is_active = sort_col == col;
        let raw = if is_active { format!("{label}{arrow}") } else { label.to_string() };
        let text = if leading {
            format!(" {:>prec$}", raw, prec = width - 1)
        } else {
            format!("{:>prec$}", raw, prec = width)
        };
        Span::styled(text, if is_active { active } else { cyan_bold })
    };

    Line::from(vec![
        span(MarketSortCol::Name,    "Commodity", 28, false),
        span(MarketSortCol::Buy,     "Buy",        9, true),
        span(MarketSortCol::BuyDiff,  "\u{394}",    6, false),
        span(MarketSortCol::Sell,    "Sell",       9, true),
        span(MarketSortCol::SellDiff, "\u{394}",   6, false),
        span(MarketSortCol::Mean,    "Mean",       9, true),
        span(MarketSortCol::Stock,   "Stock",      9, true),
        span(MarketSortCol::Demand,  "Demand",     9, true),
        Span::styled("   green=good deal  red=bad deal", Style::default().fg(Color::DarkGray)),
    ])
}

/// Absolute credit deviation of `price` from `mean` for sorting (returns i32::MIN when N/A
/// so unavailable entries sort last in ascending order).
pub(super) fn raw_diff(price: i32, mean: i32) -> i32 {
    if price <= 0 || mean <= 0 { i32::MIN } else { price - mean }
}

/// Render one commodity row with buy/sell coloured relative to mean_price.
/// `ship_cargo` and `carrier_stock` are shown next to the need count when non-zero.
pub(super) fn commodity_row(
    c: &CommodityResponse,
    todo_needed: &HashMap<String, i32>,
    ship_cargo: &HashMap<String, i32>,
    carrier_stock: &HashMap<String, i32>,
) -> Line<'static> {
    let buy_str = if c.buy_price > 0 { format!("{:>8}", c.buy_price) } else { format!("{:>8}", "-") };
    let sell_str = if c.sell_price > 0 { format!("{:>8}", c.sell_price) } else { format!("{:>8}", "-") };

    let buy_color = price_vs_mean(c.buy_price, c.mean_price, false);
    let sell_color = price_vs_mean(c.sell_price, c.mean_price, true);

    let norm = normalize_commodity_name(&c.name);
    let in_todo = todo_needed.get(&norm).copied();

    let name_style = if in_todo.is_some() {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let mut spans = vec![
        Span::styled(truncate(&c.name, 28), name_style),
        Span::styled(format!(" {}", buy_str), Style::default().fg(buy_color)),
        Span::styled(diff_vs_mean(c.buy_price, c.mean_price), Style::default().fg(buy_color)),
        Span::styled(format!(" {}", sell_str), Style::default().fg(sell_color)),
        Span::styled(diff_vs_mean(c.sell_price, c.mean_price), Style::default().fg(sell_color)),
        Span::styled(format!(" {:>8}", c.mean_price), Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {:>8}", c.stock), Style::default().fg(Color::White)),
        Span::styled(format!(" {:>8}", c.demand), Style::default().fg(Color::White)),
    ];

    if let Some(needed) = in_todo {
        let tag_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(Color::Rgb(160, 130, 0));
        spans.push(Span::styled(format!("  \u{2605} need:{}", needed), tag_style));
        let in_ship = ship_cargo.get(&norm).copied().unwrap_or(0);
        let in_carrier = carrier_stock.get(&norm).copied().unwrap_or(0);
        if in_ship > 0 {
            spans.push(Span::styled(format!("  ship:{}", in_ship), dim_style));
        }
        if in_carrier > 0 {
            spans.push(Span::styled(format!("  carrier:{}", in_carrier), dim_style));
        }
    }

    Line::from(spans)
}

/// Returns a 6-char credit-difference string vs mean, e.g. `" -800"` or `"      "` when N/A.
fn diff_vs_mean(price: i32, mean: i32) -> String {
    if price <= 0 || mean <= 0 {
        return "      ".to_string();
    }
    let diff = price - mean;
    format!("{:>+6}", diff.clamp(-99999, 99999))
}

/// Green when `price` is a good deal vs `mean`, red when it's a bad deal.
/// `higher_is_better` = true for sell price, false for buy price.
/// Neutral (white) within ±2% of mean.
fn price_vs_mean(price: i32, mean: i32, higher_is_better: bool) -> Color {
    if price <= 0 || mean <= 0 {
        return Color::White;
    }
    let ratio = price as f32 / mean as f32;
    if higher_is_better {
        if ratio >= 1.02 { Color::Green } else if ratio <= 0.98 { Color::Red } else { Color::White }
    } else {
        if ratio <= 0.98 { Color::Green } else if ratio >= 1.02 { Color::Red } else { Color::White }
    }
}

pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{s:<width$}", width = max)
    } else {
        format!("{:.width$}\u{2026}", s, width = max - 1)
    }
}

pub(super) fn fmt_ts(ts: Option<&chrono::DateTime<chrono::Utc>>) -> String {
    ts.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "\u{2014}".to_string())
}

pub(super) fn fmt_ts_short(ts: Option<&chrono::DateTime<chrono::Utc>>) -> String {
    ts.map(|t| t.format("%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "\u{2014}".to_string())
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum SearchState {
    Idle,
    Typing,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum FocusArea {
    List,
    Detail,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum StationDetailTab {
    Overview,
    Market,
    Outfitting,
    Shipyard,
}

impl StationDetailTab {
    pub(super) fn next(self) -> Option<Self> {
        match self {
            Self::Overview => Some(Self::Market),
            Self::Market => Some(Self::Outfitting),
            Self::Outfitting => Some(Self::Shipyard),
            Self::Shipyard => None,
        }
    }
    pub(super) fn prev(self) -> Option<Self> {
        match self {
            Self::Overview => None,
            Self::Market => Some(Self::Overview),
            Self::Outfitting => Some(Self::Market),
            Self::Shipyard => Some(Self::Outfitting),
        }
    }
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Market => "Market",
            Self::Outfitting => "Outfitting",
            Self::Shipyard => "Shipyard",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum CarrierDetailTab {
    Overview,
    Market,
    Outfitting,
    Shipyard,
    Inventory,
}

impl CarrierDetailTab {
    pub(super) fn next(self) -> Option<Self> {
        match self {
            Self::Overview   => Some(Self::Market),
            Self::Market     => Some(Self::Outfitting),
            Self::Outfitting => Some(Self::Shipyard),
            Self::Shipyard   => Some(Self::Inventory),
            Self::Inventory  => None,
        }
    }
    pub(super) fn prev(self) -> Option<Self> {
        match self {
            Self::Overview   => None,
            Self::Market     => Some(Self::Overview),
            Self::Outfitting => Some(Self::Market),
            Self::Shipyard   => Some(Self::Outfitting),
            Self::Inventory  => Some(Self::Shipyard),
        }
    }
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview   => "Overview",
            Self::Market     => "Market",
            Self::Outfitting => "Outfitting",
            Self::Shipyard   => "Shipyard",
            Self::Inventory  => "Inventory",
        }
    }
}

/// Compute remaining construction needs, subtracting ship cargo and carrier stock.
pub(super) fn compute_todo_needed(
    construction_items: &[crate::todo::ConstructionTodoItem],
    cargo: &[crate::journal_reader::CargoItem],
    carrier_stock: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    let mut needed: HashMap<String, i32> = HashMap::new();
    for item in construction_items {
        for res in &item.resources {
            let remaining = (res.required_amount - res.provided_amount).max(0);
            if remaining > 0 {
                *needed.entry(normalize_commodity_name(&res.commodity_name)).or_insert(0) += remaining;
            }
        }
    }
    for c in cargo {
        let norm = normalize_commodity_name(&c.name);
        if let Some(n) = needed.get_mut(&norm) {
            *n = (*n - c.count).max(0);
        }
    }
    for (norm, qty) in carrier_stock {
        if let Some(n) = needed.get_mut(norm) {
            *n = (*n - qty).max(0);
        }
    }
    needed.retain(|_, v| *v > 0);
    needed
}

/// Filter `todo_needed` to only commodities a market actually has for sale.
pub(super) fn effective_todo_for_market(
    commodities: &[CommodityResponse],
    todo_needed: &HashMap<String, i32>,
) -> HashMap<String, i32> {
    commodities.iter()
        .filter(|c| c.buy_price > 0 && c.stock > 0)
        .filter_map(|c| {
            let norm = normalize_commodity_name(&c.name);
            todo_needed.get(&norm).map(|&n| (norm, n))
        })
        .collect()
}

/// Sort commodities: todo items first, then by the chosen column.
pub(super) fn sorted_commodities<'a>(
    commodities: &'a [CommodityResponse],
    effective_todo: &HashMap<String, i32>,
    sort_col: MarketSortCol,
    sort_asc: bool,
) -> Vec<&'a CommodityResponse> {
    let mut sorted: Vec<&CommodityResponse> = commodities.iter().collect();
    sorted.sort_by(|a, b| {
        let a_norm = normalize_commodity_name(&a.name);
        let b_norm = normalize_commodity_name(&b.name);
        let group = effective_todo.contains_key(&b_norm).cmp(&effective_todo.contains_key(&a_norm));
        if group != Ordering::Equal { return group; }
        let col_ord = match sort_col {
            MarketSortCol::Name     => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            MarketSortCol::Buy      => a.buy_price.cmp(&b.buy_price),
            MarketSortCol::BuyDiff  => raw_diff(a.buy_price, a.mean_price).cmp(&raw_diff(b.buy_price, b.mean_price)),
            MarketSortCol::Sell     => a.sell_price.cmp(&b.sell_price),
            MarketSortCol::SellDiff => raw_diff(a.sell_price, a.mean_price).cmp(&raw_diff(b.sell_price, b.mean_price)),
            MarketSortCol::Mean     => a.mean_price.cmp(&b.mean_price),
            MarketSortCol::Stock    => a.stock.cmp(&b.stock),
            MarketSortCol::Demand   => a.demand.cmp(&b.demand),
        };
        if sort_asc { col_ord } else { col_ord.reverse() }
    });
    sorted
}

/// Render outfitting module list (shared by station and carrier views).
pub(super) fn outfitting_lines(modules: &[ModuleResponse]) -> Vec<Line<'static>> {
    if modules.is_empty() {
        return vec![Line::from(Span::styled("No outfitting data available.", Style::default().fg(Color::DarkGray)))];
    }
    let mut lines = Vec::new();
    let mut last_cat = String::new();
    for m in modules {
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

/// Render shipyard ship list (shared by station and carrier views).
pub(super) fn shipyard_lines(ships: &[ShipResponse]) -> Vec<Line<'static>> {
    if ships.is_empty() {
        return vec![Line::from(Span::styled("No shipyard data available.", Style::default().fg(Color::DarkGray)))];
    }
    ships.iter().map(|s| {
        let name = s.name.as_deref().unwrap_or(&s.id);
        let val = if s.basevalue > 0 { format!("{:>14}", s.basevalue) } else { format!("{:>14}", "-") };
        Line::from(format!("{:<38} {}", truncate(name, 38), val))
    }).collect()
}
