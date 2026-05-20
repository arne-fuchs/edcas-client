use std::collections::HashMap;

use edcas_common::api::CommodityResponse;
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
    BuyPct,
    Sell,
    SellPct,
    Mean,
    Stock,
    Demand,
}

impl MarketSortCol {
    pub(super) fn from_digit(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Name),
            '2' => Some(Self::Buy),
            '3' => Some(Self::BuyPct),
            '4' => Some(Self::Sell),
            '5' => Some(Self::SellPct),
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
        span(MarketSortCol::BuyPct,  "%",          6, false),
        span(MarketSortCol::Sell,    "Sell",       9, true),
        span(MarketSortCol::SellPct, "%",          6, false),
        span(MarketSortCol::Mean,    "Mean",       9, true),
        span(MarketSortCol::Stock,   "Stock",      9, true),
        span(MarketSortCol::Demand,  "Demand",     9, true),
        Span::styled("   green=good deal  red=bad deal", Style::default().fg(Color::DarkGray)),
    ])
}

/// Raw percentage deviation of `price` from `mean` (returns NEG_INFINITY when N/A so
/// unavailable entries sort last in ascending order).
pub(super) fn raw_pct(price: i32, mean: i32) -> f32 {
    if price <= 0 || mean <= 0 { f32::NEG_INFINITY } else { (price as f32 - mean as f32) / mean as f32 * 100.0 }
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
        Span::styled(pct_vs_mean(c.buy_price, c.mean_price), Style::default().fg(buy_color)),
        Span::styled(format!(" {}", sell_str), Style::default().fg(sell_color)),
        Span::styled(pct_vs_mean(c.sell_price, c.mean_price), Style::default().fg(sell_color)),
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

/// Returns a 6-char percentage string vs mean, e.g. `"  -8%"` or `"      "` when N/A.
fn pct_vs_mean(price: i32, mean: i32) -> String {
    if price <= 0 || mean <= 0 {
        return "      ".to_string();
    }
    let pct = ((price as f32 - mean as f32) / mean as f32 * 100.0).round() as i32;
    format!("{:>+5}%", pct.clamp(-999, 999))
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
