use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use edcas_common::api::ConstructionDepotResponse;
use crate::todo::{ConstructionTodoItem, ConstructionTodoResource};

// ── Public helpers (used by stations.rs) ─────────────────────────────────────

/// Render depot detail from live journal data (most up-to-date resource counts).
pub(super) fn depot_detail_lines_local(
    depot: &crate::journal_reader::ConstructionDepotData,
    ship_cargo: &std::collections::HashMap<String, i32>,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    build_detail_from_local(&mut lines, depot, ship_cargo, None);
    lines
}

/// Render depot detail from live journal data with a selected resource row.
pub(super) fn depot_detail_lines_local_with_selection(
    depot: &crate::journal_reader::ConstructionDepotData,
    ship_cargo: &std::collections::HashMap<String, i32>,
    selected_idx: usize,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    build_detail_from_local(&mut lines, depot, ship_cargo, Some(selected_idx));
    lines
}

/// Render depot detail from API response (cross-session data).
pub(super) fn build_detail_from_response(
    lines: &mut Vec<Line<'static>>,
    depot: &ConstructionDepotResponse,
    ship_cargo: &std::collections::HashMap<String, i32>,
    selected_idx: Option<usize>,
) {
    build_depot_header(lines, &depot.station_name, depot.progress, depot.construction_complete, depot.construction_failed);

    if !depot.system_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  System  ", Style::default().fg(Color::Cyan)),
            Span::styled(depot.system_name.clone(), Style::default().fg(Color::White)),
        ]));
    }
    if !depot.last_updated.is_empty() {
        let ts = depot.last_updated.get(..19).unwrap_or(&depot.last_updated);
        lines.push(Line::from(vec![
            Span::styled("  Updated ", Style::default().fg(Color::Cyan)),
            Span::styled(ts.to_string(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(section_header(&format!("Resources ({})", depot.resources.len())));

    let mut sorted = depot.resources.clone();
    sorted.sort_by(|a, b| {
        let done_a = a.provided_amount >= a.required_amount;
        let done_b = b.provided_amount >= b.required_amount;
        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
    });

    for (i, res) in sorted.iter().enumerate() {
        let frac = if res.required_amount == 0 { 1.0f32 } else { res.provided_amount as f32 / res.required_amount as f32 };
        let remaining = (res.required_amount - res.provided_amount).max(0);
        let norm = super::util::normalize_commodity_name(&res.name);
        let in_ship = ship_cargo.get(&norm).copied().unwrap_or(0);
        push_resource_line(lines, &res.display_name, res.provided_amount, res.required_amount, frac, remaining, res.payment, in_ship, selected_idx == Some(i));
    }
}

pub(super) fn construction_todo_item_from_depot(
    depot: &crate::journal_reader::ConstructionDepotData,
) -> ConstructionTodoItem {
    let resources = depot.submission.resources.iter()
        .filter(|r| r.provided_amount < r.required_amount)
        .map(|r| ConstructionTodoResource {
            commodity_name: r.name.clone(),
            display_name: r.display_name.clone(),
            required_amount: r.required_amount,
            provided_amount: r.provided_amount,
            payment: r.payment,
        })
        .collect();
    ConstructionTodoItem {
        market_id: depot.submission.market_id,
        station_name: depot.submission.station_name.clone(),
        system_name: depot.system_name.clone(),
        resources,
    }
}

pub(super) fn construction_todo_item_from_response(depot: &ConstructionDepotResponse) -> ConstructionTodoItem {
    let resources = depot.resources.iter()
        .filter(|r| r.provided_amount < r.required_amount)
        .map(|r| ConstructionTodoResource {
            commodity_name: r.name.clone(),
            display_name: r.display_name.clone(),
            required_amount: r.required_amount,
            provided_amount: r.provided_amount,
            payment: r.payment,
        })
        .collect();
    ConstructionTodoItem {
        market_id: depot.market_id,
        station_name: depot.station_name.clone(),
        system_name: depot.system_name.clone(),
        resources,
    }
}

/// Returns the number of resource rows for a given depot (for navigation bounds).
pub(super) fn depot_resource_count_local(depot: &crate::journal_reader::ConstructionDepotData) -> usize {
    depot.submission.resources.len()
}

pub(super) fn depot_resource_count_api(depot: &ConstructionDepotResponse) -> usize {
    depot.resources.len()
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn build_detail_from_local(
    lines: &mut Vec<Line<'static>>,
    local: &crate::journal_reader::ConstructionDepotData,
    ship_cargo: &std::collections::HashMap<String, i32>,
    selected_idx: Option<usize>,
) {
    let s = &local.submission;
    build_depot_header(lines, &s.station_name, s.progress, s.construction_complete, s.construction_failed);

    lines.push(Line::from(""));
    lines.push(section_header(&format!("Resources ({})", s.resources.len())));

    let mut sorted = s.resources.clone();
    sorted.sort_by(|a, b| {
        let done_a = a.provided_amount >= a.required_amount;
        let done_b = b.provided_amount >= b.required_amount;
        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
    });

    for (i, res) in sorted.iter().enumerate() {
        let frac = if res.required_amount == 0 { 1.0f32 } else { res.provided_amount as f32 / res.required_amount as f32 };
        let remaining = (res.required_amount - res.provided_amount).max(0);
        let norm = super::util::normalize_commodity_name(&res.name);
        let in_ship = ship_cargo.get(&norm).copied().unwrap_or(0);
        push_resource_line(lines, &res.display_name, res.provided_amount, res.required_amount, frac, remaining, res.payment, in_ship, selected_idx == Some(i));
    }
}

fn build_depot_header(
    lines: &mut Vec<Line<'static>>,
    station_name: &str,
    progress: f32,
    complete: bool,
    failed: bool,
) {
    let status = if complete { " ✓ Complete" } else if failed { " ✗ Failed" } else { "" };
    lines.push(Line::from(Span::styled(
        format!("{}{}", station_name, status),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    )));

    let bar_width = 30usize;
    let filled = ((progress * bar_width as f32) as usize).min(bar_width);
    let empty = bar_width - filled;
    let bar = format!("[{}{}] {:>5.1}%", "█".repeat(filled), "░".repeat(empty), progress * 100.0);
    lines.push(Line::from(Span::styled(bar, progress_color_style(progress))));
}

fn push_resource_line(
    lines: &mut Vec<Line<'static>>,
    display_name: &str,
    provided: i32,
    required: i32,
    frac: f32,
    remaining: i32,
    payment: i64,
    in_ship: i32,
    is_selected: bool,
) {
    let base_color = if frac >= 1.0 { Color::Green } else if frac > 0.0 { Color::Yellow } else { Color::DarkGray };
    let row_style = if is_selected {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let bar_w = 10usize;
    let filled = ((frac * bar_w as f32) as usize).min(bar_w);
    let empty = bar_w - filled;
    let mini_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    let payment_str = if payment > 0 {
        format!("  {:>7} cr/t", format_thousands(payment))
    } else {
        String::new()
    };

    let s = |fg: Color, text: String| -> Span<'static> {
        if is_selected { Span::styled(text, row_style) } else { Span::styled(text, Style::default().fg(fg)) }
    };

    let mut spans = vec![
        s(Color::White, format!("  {:<36}", display_name)),
        s(base_color, format!("{:>6}/{:<6}", provided, required)),
        s(base_color, format!("  {}", mini_bar)),
        s(if remaining == 0 { Color::Green } else { Color::DarkGray }, format!("  {:>6} left", remaining)),
        s(Color::DarkGray, payment_str),
    ];
    if in_ship > 0 {
        spans.push(s(Color::Rgb(160, 130, 0), format!("  ship:{}", in_ship)));
    }
    lines.push(Line::from(spans));
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("─ {} ", title),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    ))
}

fn progress_color_style(frac: f32) -> Style {
    if frac >= 1.0 {
        Style::default().fg(Color::Green)
    } else if frac >= 0.5 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Rgb(255, 140, 0))
    }
}

fn format_thousands(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}
