use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

#[derive(Clone)]
pub struct CarrierInfo {
    pub callsign: String,
    pub name: String,
    pub market_id: i64,
    pub current_system: String,
    pub body: String,
    pub distance_from_star_ls: f32,
    pub docked: bool,
    pub services: Vec<String>,
    pub fuel_level: f32,
    pub fuel_capacity: f32,
    pub bank_balance: i64,
    pub economy: String,
    pub government: String,
    pub faction: String,
    pub systems_visited: Vec<String>,
    pub last_jump_time: String,
    pub next_jump_cooldown: Option<String>,
}

#[derive(Clone)]
pub struct CarrierCommodity {
    pub name: String,
    pub buy_price: i64,
    pub sell_price: i64,
    pub stock: i64,
    pub demand: i64,
}

pub struct CarriersView {
    carriers: Vec<CarrierInfo>,
    market_data: Vec<CarrierCommodity>,
    selected_carrier_idx: usize,
    scroll_offset: usize,
}

impl CarriersView {
    pub fn new() -> Self {
        let mut view = Self {
            carriers: Vec::new(),
            market_data: Vec::new(),
            selected_carrier_idx: 0,
            scroll_offset: 0,
        };
        view.load_sample_data();
        view
    }

    fn load_sample_data(&mut self) {
        self.carriers.push(CarrierInfo {
            callsign: "J8X-44K".to_string(),
            name: "The Flying Dutchman".to_string(),
            market_id: 3700012345,
            current_system: "Sol".to_string(),
            body: "Sol A 1".to_string(),
            distance_from_star_ls: 1200.0,
            docked: true,
            services: vec![
                "Dock".to_string(), "Market".to_string(), "Black Market".to_string(),
                "Contacts".to_string(), "Mission Generator".to_string(), "Missions".to_string(),
                "Refuel".to_string(), "Repair".to_string(), "Restock".to_string(),
                "Outfitting".to_string(), "Crew Lounge".to_string(), "Shipyard".to_string(),
                "Bar".to_string(), "Pioneer Supplies".to_string(),
            ],
            fuel_level: 750.0,
            fuel_capacity: 1000.0,
            bank_balance: 5_234_892_100,
            economy: "Colony".to_string(),
            government: "Democracy".to_string(),
            faction: "Pilots Federation".to_string(),
            systems_visited: vec![
                "Sol".to_string(), "Alpha Centauri".to_string(), "Shinrarta Dezhra".to_string(),
                "Colonia".to_string(),
            ],
            last_jump_time: "2026-04-28T14:32:00Z".to_string(),
            next_jump_cooldown: Some("3h 28m".to_string()),
        });

        self.carriers.push(CarrierInfo {
            callsign: "M9A-12B".to_string(),
            name: "Deep Space Outpost".to_string(),
            market_id: 3700098765,
            current_system: "Sirius".to_string(),
            body: "Sirius A 4".to_string(),
            distance_from_star_ls: 890.0,
            docked: false,
            services: vec![
                "Dock".to_string(), "Market".to_string(), "Refuel".to_string(),
                "Repair".to_string(), "Restock".to_string(),
            ],
            fuel_level: 320.0,
            fuel_capacity: 1000.0,
            bank_balance: 1_890_450_230,
            economy: "None".to_string(),
            government: "None".to_string(),
            faction: "Pilots Federation".to_string(),
            systems_visited: vec![
                "Sirius".to_string(), "Epsilon Eridani".to_string(), "Tau Ceti".to_string(),
            ],
            last_jump_time: "2026-04-27T09:15:00Z".to_string(),
            next_jump_cooldown: Some("8h 45m".to_string()),
        });

        self.market_data = vec![
            CarrierCommodity { name: "Hydrogen Fuel".to_string(), buy_price: 86, sell_price: 0, stock: 24903, demand: 0 },
            CarrierCommodity { name: "Mineral Oil".to_string(), buy_price: 382, sell_price: 320, stock: 12456, demand: 890 },
            CarrierCommodity { name: "Food".to_string(), buy_price: 387, sell_price: 325, stock: 18234, demand: 2340 },
            CarrierCommodity { name: "Water".to_string(), buy_price: 109, sell_price: 0, stock: 45000, demand: 0 },
            CarrierCommodity { name: "Emergency Power Cells".to_string(), buy_price: 3415, sell_price: 2950, stock: 890, demand: 120 },
        ];
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if self.carriers.is_empty() {
            lines.push(Line::from("No fleet carriers tracked. Fleet carriers will appear when detected via CarrierJump events."));
            return lines;
        }

        for (idx, carrier) in self.carriers.iter().enumerate() {
            let selected_marker = if idx == self.selected_carrier_idx { ">> " } else { "   " };

            lines.push(Line::from(Span::styled(
                format!("{}Carrier: {} ({})", selected_marker, carrier.name, carrier.callsign),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("  Market ID: {}", carrier.market_id)));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Location",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(format!("  System: {}, Body: {}", carrier.current_system, carrier.body)));
            lines.push(Line::from(format!("  Distance from Star: {:.1} Ls", carrier.distance_from_star_ls)));
            lines.push(Line::from(format!("  Docked: {}", bool_icon(carrier.docked))));
            lines.push(Line::from(format!("  Last Jump: {}", carrier.last_jump_time)));
            if let Some(ref cooldown) = carrier.next_jump_cooldown {
                lines.push(Line::from(format!("  Jump Cooldown: {}", cooldown)));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Carrier Status",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            let fuel_percent = (carrier.fuel_level / carrier.fuel_capacity) * 100.0;
            let fuel_bar = fuel_bar(carrier.fuel_level / carrier.fuel_capacity);
            let fuel_color = if fuel_percent > 50.0 { Color::Green } else if fuel_percent > 20.0 { Color::Yellow } else { Color::Red };
            lines.push(Line::from(Span::styled(
                format!("  Fuel: {:.0}/{:.0} ({:.0}%) {}", carrier.fuel_level, carrier.fuel_capacity, fuel_percent, fuel_bar),
                Style::default().fg(fuel_color),
            )));
            lines.push(Line::from(format!("  Bank Balance: {} Cr", format_thousands(carrier.bank_balance))));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Services",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            for chunk in carrier.services.chunks(4) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Market",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {:<25} {:>10} {:>10} {:>10} {:>10}", "Item", "Buy", "Sell", "Stock", "Demand"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "  ".to_string() + &"─".repeat(65),
                Style::default().fg(Color::DarkGray),
            )));
            for item in &self.market_data {
                lines.push(Line::from(format!("  {:<25} {:>10} {:>10} {:>10} {:>10}",
                    item.name,
                    format_price(item.buy_price),
                    if item.sell_price > 0 { format_price(item.sell_price) } else { "N/A".to_string() },
                    format_number(item.stock),
                    if item.demand > 0 { format_number(item.demand) } else { "N/A".to_string() },
                )));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Jump History",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(format!("  Systems Visited: {}", carrier.systems_visited.join(" -> "))));
            lines.push(Line::from(""));
        }
        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_carrier_idx > 0 {
                    self.selected_carrier_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                let max_idx = self.carriers.len().saturating_sub(1);
                if self.selected_carrier_idx < max_idx {
                    self.selected_carrier_idx += 1;
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self.build_lines();

        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;

        let mut paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Carriers ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );

        if content_height > visible_height {
            let max_scroll = content_height.saturating_sub(visible_height);
            paragraph = paragraph.scroll((self.scroll_offset.min(max_scroll) as u16, 0));
        }

        frame.render_widget(paragraph, area);
    }
}

fn bool_icon(val: bool) -> &'static str {
    if val { "Yes" } else { "No" }
}

fn format_price(price: i64) -> String {
    if price >= 1_000_000 {
        format!("{:.1}M", price as f64 / 1_000_000.0)
    } else if price >= 1_000 {
        format!("{:.1}K", price as f64 / 1_000.0)
    } else {
        price.to_string()
    }
}

fn format_number(num: i64) -> String {
    if num >= 1_000_000_000 {
        format!("{:.1}B", num as f64 / 1_000_000_000.0)
    } else if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

fn fuel_bar(ratio: f32) -> String {
    let filled = (ratio * 10.0).round() as usize;
    let empty = 10 - filled.min(10);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
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
