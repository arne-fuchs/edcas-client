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
pub struct StationInfo {
    pub name: String,
    pub station_type: String,
    pub system_name: String,
    pub market_id: i64,
    pub distance_from_star_ls: f32,
    pub faction: String,
    pub government: String,
    pub economy: String,
    pub economies: Vec<EconomyInfo>,
    pub allegiance: String,
    pub services: Vec<String>,
    pub landing_pads: Option<LandingPads>,
}

#[derive(Clone)]
pub struct EconomyInfo {
    pub name: String,
    pub proportion: f32,
}

#[derive(Clone)]
pub struct LandingPads {
    pub small: i32,
    pub medium: i32,
    pub large: i32,
}

#[derive(Clone)]
pub struct CommodityInfo {
    pub name: String,
    pub buy_price: i64,
    pub sell_price: i64,
    pub stock: i64,
    pub demand: i64,
    pub stock_bracket: i32,
    pub demand_bracket: i32,
}

pub struct StationsView {
    stations: Vec<StationInfo>,
    market_data: Vec<CommodityInfo>,
    selected_station_idx: usize,
    scroll_offset: usize,
}

impl StationsView {
    pub fn new() -> Self {
        let mut view = Self {
            stations: Vec::new(),
            market_data: Vec::new(),
            selected_station_idx: 0,
            scroll_offset: 0,
        };
        view.load_sample_data();
        view
    }

    fn load_sample_data(&mut self) {
        self.stations.push(StationInfo {
            name: "Jameson Memorial".to_string(),
            station_type: "Orbis Starport".to_string(),
            system_name: "Shinrarta Dezhra".to_string(),
            market_id: 322897632,
            distance_from_star_ls: 893.45,
            faction: "Shinrarta Industry Inc".to_string(),
            government: "Democracy".to_string(),
            economy: "High Tech".to_string(),
            economies: vec![
                EconomyInfo { name: "High Tech".to_string(), proportion: 0.70 },
                EconomyInfo { name: "Refinery".to_string(), proportion: 0.30 },
            ],
            allegiance: "Independent".to_string(),
            services: vec![
                "Dock".to_string(), "Autodock".to_string(), "Market".to_string(),
                "Black Market".to_string(), "Contacts".to_string(), "Universal Cartographics".to_string(),
                "Mission Generator".to_string(), "Missions".to_string(), "Tuning".to_string(),
                "Workshop".to_string(), "Material Trader".to_string(), "Search and Rescue".to_string(),
                "Refuel".to_string(), "Repair".to_string(), "Restock".to_string(),
                "Shipyard".to_string(), "Outfitting".to_string(), "Crew Lounge".to_string(),
                "Interstellar Factors Contact".to_string(),
            ],
            landing_pads: Some(LandingPads { small: 20, medium: 12, large: 4 }),
        });

        self.market_data = vec![
            CommodityInfo { name: "Explosives".to_string(), buy_price: 572, sell_price: 492, stock: 17248, demand: 310, stock_bracket: 3, demand_bracket: 1 },
            CommodityInfo { name: "Mineral Oil".to_string(), buy_price: 645, sell_price: 554, stock: 23929, demand: 582, stock_bracket: 3, demand_bracket: 1 },
            CommodityInfo { name: "Hydrogen Fuel".to_string(), buy_price: 126, sell_price: 114, stock: 41202, demand: 11442, stock_bracket: 3, demand_bracket: 3 },
            CommodityInfo { name: "Survival Equipment".to_string(), buy_price: 3415, sell_price: 2952, stock: 3335, demand: 757, stock_bracket: 2, demand_bracket: 1 },
            CommodityInfo { name: "Clothing".to_string(), buy_price: 762, sell_price: 658, stock: 15522, demand: 3147, stock_bracket: 3, demand_bracket: 1 },
            CommodityInfo { name: "Consumer Technology".to_string(), buy_price: 7493, sell_price: 6926, stock: 2590, demand: 861, stock_bracket: 2, demand_bracket: 1 },
            CommodityInfo { name: "Hardware".to_string(), buy_price: 750, sell_price: 648, stock: 21225, demand: 1069, stock_bracket: 3, demand_bracket: 2 },
            CommodityInfo { name: "Food".to_string(), buy_price: 608, sell_price: 525, stock: 25917, demand: 3859, stock_bracket: 3, demand_bracket: 1 },
            CommodityInfo { name: "Imperial Slaves".to_string(), buy_price: 18492, sell_price: 17167, stock: 478, demand: 244, stock_bracket: 1, demand_bracket: 1 },
            CommodityInfo { name: "Slaves".to_string(), buy_price: 13209, sell_price: 12262, stock: 592, demand: 282, stock_bracket: 1, demand_bracket: 1 },
        ];
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if let Some(station) = self.stations.first() {
            lines.push(Line::from(Span::styled(
                format!("Station: {}", station.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("System: {}", station.system_name)));
            lines.push(Line::from(format!("Type: {}", station.station_type)));
            lines.push(Line::from(format!("Market ID: {}", station.market_id)));
            lines.push(Line::from(format!("Distance from Star: {:.2} Ls", station.distance_from_star_ls)));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Station Details",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(format!("  Faction: {}", station.faction)));
            lines.push(Line::from(format!("  Government: {}", station.government)));
            lines.push(Line::from(format!("  Allegiance: {}", station.allegiance)));
            lines.push(Line::from(format!("  Primary Economy: {}", station.economy)));
            if !station.economies.is_empty() {
                for econ in &station.economies {
                    lines.push(Line::from(format!("    {} ({:.0}%)", econ.name, econ.proportion * 100.0)));
                }
            }
            lines.push(Line::from(""));

            if let Some(ref pads) = station.landing_pads {
                lines.push(Line::from(Span::styled(
                    "Landing Pads",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                )));
                lines.push(Line::from(format!("  Small: {}, Medium: {}, Large: {}", pads.small, pads.medium, pads.large)));
                lines.push(Line::from(""));
            }

            lines.push(Line::from(Span::styled(
                "Services",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            let services_per_line = 4;
            for chunk in station.services.chunks(services_per_line) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Market Data",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {:<25} {:>10} {:>10} {:>10} {:>10}", "Commodity", "Buy", "Sell", "Stock", "Demand"),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "  ".to_string() + &"─".repeat(65),
                Style::default().fg(Color::DarkGray),
            )));
            for commodity in &self.market_data {
                lines.push(Line::from(format!("  {:<25} {:>10} {:>10} {:>10} {:>10}",
                    commodity.name,
                    format_price(commodity.buy_price),
                    format_price(commodity.sell_price),
                    format_number(commodity.stock),
                    format_number(commodity.demand),
                )));
            }
        } else {
            lines.push(Line::from("No station data available. Dock at a station to view information."));
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_station_idx > 0 {
                    self.selected_station_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                let max_idx = self.stations.len().saturating_sub(1);
                if self.selected_station_idx < max_idx {
                    self.selected_station_idx += 1;
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
                    .title(" Stations ")
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
