use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{JournalData, SuitData, SuitWeapon};
use crate::views::ViewEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const COMBAT_RANKS: &[&str] = &[
    "Harmless", "Mostly Harmless", "Novice", "Competent", "Expert",
    "Master", "Dangerous", "Deadly", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];
const TRADE_RANKS: &[&str] = &[
    "Penniless", "Mostly Penniless", "Peddler", "Dealer", "Merchant",
    "Broker", "Entrepreneur", "Tycoon", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];
const EXPLORE_RANKS: &[&str] = &[
    "Aimless", "Mostly Aimless", "Scout", "Surveyor", "Trailblazer",
    "Pathfinder", "Ranger", "Pioneer", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];
const SOLDIER_RANKS: &[&str] = &[
    "Defenceless", "Mostly Defenceless", "Rookie", "Soldier", "Gunslinger",
    "Warrior", "Gladiator", "Deadeye", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];
const EXOBIO_RANKS: &[&str] = &[
    "Directionless", "Mostly Directionless", "Compiler", "Collector", "Cataloguer",
    "Taxonomist", "Ecologist", "Geneticist", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];
const FEDERATION_RANKS: &[&str] = &[
    "None", "Recruit", "Cadet", "Midshipman", "Petty Officer", "Chief Petty Officer",
    "Warrant Officer", "Ensign", "Lieutenant", "Lt. Commander", "Post Commander",
    "Post Captain", "Rear Admiral", "Vice Admiral", "Admiral",
];
const EMPIRE_RANKS: &[&str] = &[
    "None", "Outsider", "Serf", "Master", "Squire", "Knight",
    "Lord", "Baron", "Viscount", "Count", "Earl", "Marquis", "Duke", "Prince", "King",
];
const CQC_RANKS: &[&str] = &[
    "Helpless", "Mostly Helpless", "Amateur", "Semi Professional", "Professional",
    "Champion", "Hero", "Legend", "Elite", "Elite I", "Elite II", "Elite III", "Elite IV", "Elite V",
];

fn rank_name(ranks: &'static [&'static str], value: u8) -> &'static str {
    ranks.get(value as usize).copied().unwrap_or("Unknown")
}

fn progress_bar(percent: u8, width: usize) -> String {
    let filled = (percent as usize * width / 100).min(width);
    let empty = width - filled;
    format!("[{}{}] {:>3}%", "█".repeat(filled), "░".repeat(empty), percent)
}

fn format_credits(n: i64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B CR", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M CR", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K CR", n as f64 / 1_000.0)
    } else {
        format!("{n} CR")
    }
}

fn reputation_bar(value: f32, width: usize) -> String {
    // Reputation is -100 to +100; centre the bar at 0
    let clamped = value.clamp(-100.0, 100.0);
    let filled = ((clamped + 100.0) / 200.0 * width as f32) as usize;
    let empty = width - filled;
    format!("[{}{}] {:>+.0}", "█".repeat(filled), "░".repeat(empty), value)
}

pub struct PilotView {
    scroll: u16,
}

impl PilotView {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.scroll += 1;
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.scroll += 10;
            }
            _ => return ViewEvent::None,
        }
        ViewEvent::Consumed
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let lines = self.build_lines(journal);
        let inner_height = area.height.saturating_sub(2) as usize;
        let max_scroll = lines.len().saturating_sub(inner_height) as u16;
        self.scroll = self.scroll.min(max_scroll);

        frame.render_widget(
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Pilot ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(255, 140, 0))),
                )
                .scroll((self.scroll, 0)),
            area,
        );
    }

    fn build_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let p = &journal.pilot;
        let mut lines: Vec<Line<'static>> = Vec::new();

        if p.name.is_empty() {
            lines.push(Line::from(Span::styled(
                " No pilot data — ensure journal directory is configured and game is running",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        let orange = Style::default().fg(Color::Rgb(255, 140, 0));
        let cyan = Style::default().fg(Color::Cyan);
        let white = Style::default().fg(Color::White);
        let bold = Style::default().fg(Color::White).add_modifier(Modifier::BOLD);
        let gray = Style::default().fg(Color::DarkGray);

        let section = |title: &'static str| -> Line<'static> {
            Line::from(vec![
                Span::raw(" "),
                Span::styled(title, Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
            ])
        };
        let divider = || Line::from(Span::styled("─".repeat(60), gray));
        let blank = || Line::from("");

        // ── Commander ────────────────────────────────────────────────
        lines.push(blank());
        lines.push(section("COMMANDER"));
        lines.push(divider());
        lines.push(Line::from(vec![
            Span::styled("  Name        ", cyan),
            Span::styled(p.name.clone(), bold),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Credits     ", cyan),
            Span::styled(format_credits(p.credits), orange),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Game Mode   ", cyan),
            Span::styled(p.game_mode.clone(), white),
        ]));
        let expansions = match (p.horizons, p.odyssey) {
            (true, true) => "Horizons + Odyssey",
            (true, false) => "Horizons",
            (false, true) => "Odyssey",
            (false, false) => "Base",
        };
        lines.push(Line::from(vec![
            Span::styled("  Expansions  ", cyan),
            Span::styled(expansions, white),
        ]));

        // ── Ranks ────────────────────────────────────────────────────
        lines.push(blank());
        lines.push(section("RANKS"));
        lines.push(divider());

        let rank_line = |label: &'static str, rank_names: &'static [&'static str], rank: u8, prog: u8| -> Line<'static> {
            let name = rank_name(rank_names, rank).to_string();
            Line::from(vec![
                Span::styled(format!("  {label:<16}", label = label), cyan),
                Span::styled(format!("{:<22}", name), white),
                Span::styled(progress_bar(prog, 16), gray),
            ])
        };

        lines.push(rank_line("Combat", COMBAT_RANKS, p.rank_combat, p.progress_combat));
        lines.push(rank_line("Trade", TRADE_RANKS, p.rank_trade, p.progress_trade));
        lines.push(rank_line("Exploration", EXPLORE_RANKS, p.rank_explore, p.progress_explore));
        lines.push(rank_line("CQC", CQC_RANKS, p.rank_cqc, p.progress_cqc));
        if p.odyssey {
            lines.push(rank_line("Soldier", SOLDIER_RANKS, p.rank_soldier, p.progress_soldier));
            lines.push(rank_line("Exobiologist", EXOBIO_RANKS, p.rank_exobiologist, p.progress_exobiologist));
        }
        lines.push(rank_line("Federation", FEDERATION_RANKS, p.rank_federation, p.progress_federation));
        lines.push(rank_line("Empire", EMPIRE_RANKS, p.rank_empire, p.progress_empire));

        // ── Reputations ──────────────────────────────────────────────
        lines.push(blank());
        lines.push(section("REPUTATION"));
        lines.push(divider());

        let rep_line = |label: &'static str, value: f32| -> Line<'static> {
            Line::from(vec![
                Span::styled(format!("  {label:<16}", label = label), cyan),
                Span::styled(reputation_bar(value, 24), gray),
            ])
        };

        lines.push(rep_line("Empire", p.reputation_empire));
        lines.push(rep_line("Federation", p.reputation_federation));
        lines.push(rep_line("Alliance", p.reputation_alliance));

        // ── Powerplay ────────────────────────────────────────────────
        if !p.power.is_empty() {
            lines.push(blank());
            lines.push(section("POWERPLAY"));
            lines.push(divider());
            lines.push(Line::from(vec![
                Span::styled("  Pledged To  ", cyan),
                Span::styled(p.power.clone(), bold),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Merits      ", cyan),
                Span::styled(format!("{}", p.power_merits), orange),
            ]));
        }

        lines.push(blank());

        // ── Suit ─────────────────────────────────────────────────────
        if p.odyssey {
            lines.push(section("SUIT"));
            lines.push(divider());
            match &p.suit {
                None => {
                    lines.push(Line::from(Span::styled(
                        "  No suit data — requires a SuitLoadout event",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                Some(suit) => {
                    suit_lines(suit, &mut lines);
                }
            }
            lines.push(blank());
        }

        lines
    }
}

fn suit_lines(suit: &SuitData, lines: &mut Vec<Line<'static>>) {
    let orange = Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
    let cyan   = Style::default().fg(Color::Cyan);
    let white  = Style::default().fg(Color::White);
    let purple = Style::default().fg(Color::Rgb(180, 100, 255));
    let dim    = Style::default().fg(Color::DarkGray);

    let grade_dots = "●".repeat(suit.grade as usize) + &"○".repeat(5usize.saturating_sub(suit.grade as usize));
    lines.push(Line::from(vec![
        Span::styled("  Type        ", cyan),
        Span::styled(
            format!("{} G{}  {}", suit.suit_type, suit.grade, grade_dots),
            orange,
        ),
    ]));

    if !suit.loadout_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Loadout     ", cyan),
            Span::styled(suit.loadout_name.clone(), white.add_modifier(Modifier::BOLD)),
        ]));
    }

    if !suit.mods.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Suit Mods   ", cyan),
            Span::styled(suit.mods.join(", "), purple),
        ]));
    }

    if suit.weapons.is_empty() {
        lines.push(Line::from(Span::styled("  No weapons in loadout", dim)));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Weapons", cyan)));
        for w in &suit.weapons {
            weapon_line(w, lines);
        }
    }
}

fn weapon_line(w: &SuitWeapon, lines: &mut Vec<Line<'static>>) {
    let cyan   = Style::default().fg(Color::Cyan);
    let white  = Style::default().fg(Color::White);
    let purple = Style::default().fg(Color::Rgb(180, 100, 255));
    let dim    = Style::default().fg(Color::DarkGray);

    let grade_dots = "●".repeat(w.class as usize) + &"○".repeat(5usize.saturating_sub(w.class as usize));
    lines.push(Line::from(vec![
        Span::styled(format!("    {:<18}", w.slot), cyan),
        Span::styled(format!("{:<24}", w.name), white),
        Span::styled(format!("G{}  {}", w.class, grade_dots), purple),
    ]));
    if !w.mods.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("      mods: ", dim),
            Span::styled(w.mods.join(", "), purple),
        ]));
    }
}
