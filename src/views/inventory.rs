use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::journal_reader::{InventoryItem, JournalData, OnFootInventory};
use crate::views::ViewEvent;

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Materials,
    Cargo,
    Backpack,
    ShipLocker,
}

impl Tab {
    fn next(self) -> Option<Self> {
        match self {
            Self::Materials => Some(Self::Cargo),
            Self::Cargo => Some(Self::Backpack),
            Self::Backpack => Some(Self::ShipLocker),
            Self::ShipLocker => None,
        }
    }
    fn prev(self) -> Option<Self> {
        match self {
            Self::Materials => None,
            Self::Cargo => Some(Self::Materials),
            Self::Backpack => Some(Self::Cargo),
            Self::ShipLocker => Some(Self::Backpack),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Materials => "Materials",
            Self::Cargo => "Cargo",
            Self::Backpack => "Backpack",
            Self::ShipLocker => "Ship Locker",
        }
    }
}

pub struct InventoryView {
    tab: Tab,
    scroll: usize,
}

impl InventoryView {
    pub fn new() -> Self {
        Self { tab: Tab::Materials, scroll: 0 }
    }

    fn build_header_lines(&self) -> Vec<Line<'static>> {
        let tab_active = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [Tab::Materials, Tab::Cargo, Tab::Backpack, Tab::ShipLocker];
        let spans: Vec<Span> = tabs.iter().flat_map(|&t| {
            let style = if t == self.tab { tab_active } else { tab_inactive };
            [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
        }).collect();
        vec![Line::from(spans)]
    }

    fn build_body_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        match self.tab {
            Tab::Materials => self.materials_lines(journal),
            Tab::Cargo => self.cargo_lines(journal),
            Tab::Backpack => self.onfoot_lines(&journal.backpack, "Backpack"),
            Tab::ShipLocker => self.onfoot_lines(&journal.shiplocker, "Ship Locker"),
        }
    }

    fn materials_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let sections = [
            ("Raw", &journal.materials_raw, Color::Green),
            ("Manufactured", &journal.materials_manufactured, Color::Yellow),
            ("Encoded", &journal.materials_encoded, Color::Cyan),
        ];

        for (label, items, color) in &sections {
            if items.is_empty() {
                continue;
            }
            lines.push(Line::from(Span::styled(
                format!("── {label} ──"),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {:<28} {:>9}  {}", "Material", "Count", "Progress"),
                Style::default().fg(Color::DarkGray),
            )));
            let mut sorted: Vec<&InventoryItem> = items.iter().collect();
            sorted.sort_by(|a, b| b.count.cmp(&a.count));
            for item in sorted {
                let name = if item.localised.is_empty() { &item.name } else { &item.localised };
                let max = material_cap(&item.name);
                let count = item.count.min(max);
                let ratio = count as f32 / max as f32;
                let filled = (ratio * 20.0).round() as usize;
                let bar_color = if ratio < 0.25 {
                    Color::Red
                } else if ratio < 0.75 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                let bar_filled = "█".repeat(filled);
                let bar_empty = "░".repeat(20 - filled);
                let count_str = format!("{:>4}/{:<4}", count, max);
                lines.push(Line::from(vec![
                    Span::raw(format!("  {:<28} {} [", truncate_name(name, 28), count_str)),
                    Span::styled(bar_filled, Style::default().fg(bar_color)),
                    Span::styled(bar_empty, Style::default().fg(Color::DarkGray)),
                    Span::raw("]"),
                ]));
            }
            lines.push(Line::from(""));
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "No materials data. The Materials event is written to the journal at startup.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines
    }

    fn cargo_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        if journal.cargo.is_empty() {
            return vec![Line::from(Span::styled(
                "Cargo hold is empty.",
                Style::default().fg(Color::DarkGray),
            ))];
        }
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            format!("{:<32} {:>6} {:>6}", "Item", "Count", "Stolen"),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "─".repeat(46),
            Style::default().fg(Color::DarkGray),
        )));
        let total: i32 = journal.cargo.iter().map(|c| c.count).sum();
        for item in &journal.cargo {
            let name = if item.localised.is_empty() { &item.name } else { &item.localised };
            let stolen_col = if item.stolen > 0 {
                format!("{:>6}", item.stolen)
            } else {
                format!("{:>6}", "-")
            };
            lines.push(Line::from(format!("{:<32} {:>6} {}", name, item.count, stolen_col)));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Total: {total} t"),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));
        lines
    }

    fn onfoot_lines(&self, inv: &OnFootInventory, _label: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let sections = [
            ("Items", &inv.items, Color::Green),
            ("Components", &inv.components, Color::Yellow),
            ("Consumables", &inv.consumables, Color::Magenta),
            ("Data", &inv.data, Color::Cyan),
        ];
        let mut any = false;
        for (section_label, items, color) in &sections {
            if items.is_empty() {
                continue;
            }
            any = true;
            lines.push(Line::from(Span::styled(
                format!("── {section_label} ──"),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {:<36} {:>6}", "Name", "Count"),
                Style::default().fg(Color::DarkGray),
            )));
            for item in items.iter() {
                let name = if item.localised.is_empty() { &item.name } else { &item.localised };
                lines.push(Line::from(format!("  {:<36} {:>6}", name, item.count)));
            }
            lines.push(Line::from(""));
        }
        if !any {
            lines.push(Line::from(Span::styled(
                "Empty.",
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.scroll += 1;
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if let Some(next) = self.tab.next() {
                    self.tab = next;
                    self.scroll = 0;
                }
            }
            KeyCode::Char('a') | KeyCode::Left => {
                if let Some(prev) = self.tab.prev() {
                    self.tab = prev;
                    self.scroll = 0;
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let block = Block::default()
            .title(" Inventory (a/d: tabs, w/s: scroll) ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header = self.build_header_lines();
        let header_h = header.len() as u16;

        let split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_h), Constraint::Min(0)])
            .split(inner);

        frame.render_widget(Paragraph::new(header), split[0]);

        let body = self.build_body_lines(journal);
        let body_height = split[1].height as usize;
        let max_scroll = body.len().saturating_sub(body_height);

        frame.render_widget(
            Paragraph::new(body).scroll((self.scroll.min(max_scroll) as u16, 0)),
            split[1],
        );
    }
}

fn truncate_name(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        format!("{s:<max$}")
    } else {
        let truncated: String = chars[..max - 1].iter().collect();
        format!("{truncated}…")
    }
}

fn material_cap(name: &str) -> i32 {
    match name {
        // ── Raw G1 (cap 300) ──
        "carbon" | "iron" | "lead" | "nickel" | "phosphorus" | "rhenium" | "sulphur" => 300,
        // ── Raw G2 (cap 250) ──
        "arsenic" | "chromium" | "germanium" | "manganese" | "vanadium" | "zinc" | "zirconium" => 250,
        // ── Raw G3 (cap 200) ──
        "boron" | "cadmium" | "mercury" | "molybdenum" | "niobium" | "tin" | "tungsten" => 200,
        // ── Raw G4 (cap 150) ──
        "antimony" | "indium" | "palladium" | "polonium" | "ruthenium" | "selenium" | "technetium" | "tellurium" | "yttrium" => 150,
        // ── Raw G5 (cap 100) ──
        "technetiummagnetics" | "protoradiolicalloy" | "protolightalloy" | "protoheatsink" => 100,

        // ── Manufactured G1 (cap 300) ──
        "basicconductors" | "chemicalstorageunits" | "compactcomposites" | "crystalshards"
        | "gridresistors" | "heatconductionwiring" | "mechanicalscrap" | "salvagedalloysv1"
        | "temperedalloys" | "wornshieldemitters" => 300,
        // ── Manufactured G2 (cap 250) ──
        "chemicalprocessors" | "conductivecomponents" | "filamentcomposites"
        | "flawedfocuscrystals" | "galvanisingalloys" | "heatdispersionplate"
        | "heatresistantceramics" | "hybridcapacitors" | "mechanicalequipment"
        | "shieldemitters" | "uncutfocuscrystals" | "irregularcomponents" => 250,
        // ── Manufactured G3 (cap 200) ──
        "chemicaldistillery" | "conductiveceramics" | "electrochemicalarrays"
        | "focuscrystals" | "heatexchangers" | "highdensitycomposites"
        | "mechanicalcomponents" | "phasealloys" | "precipitatedalloys"
        | "refinedfocuscrystals" | "shieldingsensors" | "unknownfragment" => 200,
        // ── Manufactured G4 (cap 150) ──
        "chemicalmanipulators" | "compoundshielding" | "conductivepolymers"
        | "configurablecomponents" | "heatvanes" | "polymercapacitors"
        | "propulsionelements" | "protolightalloys" | "refinedalloys"
        | "thermicalloys" | "wornalloys" | "biotechconductors" => 150,
        // ── Manufactured G5 (cap 100) ──
        "biotechconductors2" | "compressedliquid" | "coredynamicscomposites"
        | "fedcorecomposites" | "fedproprietarycomposites" | "imperialshielding"
        | "improvisedfittings" | "militarygrades" | "militarygradealloys"
        | "militarysupercapacitors" | "pharmaceuticalisolators"
        | "proprietarycomposites" | "protoheatradiators" | "protoradiolicalloys"
        | "securecryptographickey" | "unknownenergycell"
        | "unknownorganiccircuitry" | "unknowntechnologycomponents" => 100,

        // ── Encoded G1 (cap 300) ──
        "bulkscandata" | "disruptedwakeechoes" | "legacyfirmware"
        | "shieldcyclerecordings" | "shieldsoakanalysis" | "wakesolutions" => 300,
        // ── Encoded G2 (cap 250) ──
        "abberantshieldpatternanalysis" | "abnormalcompactEmissionsdata"
        | "adaptiveencryptors" | "consumerfirmware" | "fsdtelemetry"
        | "inconsistentshieldsoakanalysis" | "jammingcodes" | "scanarchives"
        | "scrambledemissiondata" | "shieldfrequencydata"
        | "symmetrickeys" | "unusualencryptedfiles" => 250,
        // ── Encoded G3 (cap 200) ──
        "classifiedscandata" | "classifiedscanfragment" | "decodedemissiondata"
        | "differencedscanner" | "emissiondata" | "encryptedfiles"
        | "encryptioncodes" | "industrialfirmware" | "iotdatabank"
        | "largecapacitypowerregulator" | "modifiedconsumerfirmware"
        | "scandatabanks" | "securityfirmware"
        | "shieldpatternanalysis" | "wakescans" => 200,
        // ── Encoded G4 (cap 150) ──
        "adaptiveencryptors2" | "ancientbiologicaldata" | "ancientculturaldata"
        | "ancienthistoricaldata" | "ancientlanguagedata" | "ancienttechnologicaldata"
        | "archivedemissiondata" | "atypicaldisruptedwakeechoes"
        | "atypicalencryptionarchives" | "classifiedscandata2"
        | "dataportfolio" | "encryptionarchives" | "encryptionkeys"
        | "hyperspacetrajectories" | "legacyfirmware2" | "lostandfoundartefacts"
        | "patternepsilonobeliskdata" | "patternalphaobeliskdata"
        | "patternbetaobeliskdata" | "patterngammaobeliskdata"
        | "patternzetaobeliskdata" | "quirkycomponentdata"
        | "scandatabanks2" | "securityfirmware2" | "shieldsoakanalysis2"
        | "taggedencryptioncodes" | "unclassifiedrelic" => 150,
        // ── Encoded G5 (cap 100) ──
        "adaptiveencryptors3" | "classifiedscandata3" | "patterndeltaobeliskdata"
        | "specialisedlegacyfirmware" | "unknownenergysource"
        | "unknownshipsignature" | "unknownwakescan" => 100,

        // fallback
        _ => 150,
    }
}
