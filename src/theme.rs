//! UI theming. The accent colour (titles, borders, focus highlights, gauges) was historically
//! a hardcoded EDCAS orange in every view; it is now driven by `settings.appearance.color`
//! through a process-global so views can read it without threading `Settings` everywhere.

use ratatui::style::Color;
use std::sync::RwLock;

/// The historical EDCAS accent (orange). Used as the default whenever no colour is configured
/// or the configured value can't be parsed, so the out-of-the-box look is unchanged.
pub const DEFAULT_ACCENT: (u8, u8, u8) = (255, 140, 0);

static ACCENT: RwLock<(u8, u8, u8)> = RwLock::new(DEFAULT_ACCENT);

/// The current UI accent colour. Read by every view in place of a hardcoded orange.
pub fn accent() -> Color {
    let (r, g, b) = *ACCENT.read().unwrap();
    Color::Rgb(r, g, b)
}

/// Sets the accent from a configured colour string: a named colour (e.g. `orange`, `purple`),
/// a `#rrggbb` hex value, or an `r,g,b` triplet. Empty or unrecognised values fall back to the
/// [`DEFAULT_ACCENT`] so the accent is never left in an unreadable state.
pub fn set_accent(spec: &str) {
    let rgb = parse_rgb(spec).unwrap_or(DEFAULT_ACCENT);
    *ACCENT.write().unwrap() = rgb;
}

fn parse_rgb(spec: &str) -> Option<(u8, u8, u8)> {
    let s = spec.trim().to_lowercase();
    match s.as_str() {
        "" => None,
        "orange" => Some(DEFAULT_ACCENT),
        // Matches the purple already used in the Suit/Pilot/Modules views for consistency.
        "purple" => Some((180, 100, 255)),
        "white" => Some((255, 255, 255)),
        "red" => Some((255, 80, 80)),
        "green" => Some((0, 200, 80)),
        "blue" => Some((80, 140, 255)),
        "yellow" => Some((255, 215, 0)),
        "cyan" => Some((0, 200, 200)),
        "magenta" | "pink" => Some((255, 90, 200)),
        "teal" => Some((0, 180, 180)),
        _ => parse_hex_or_triplet(&s),
    }
}

fn parse_hex_or_triplet(s: &str) -> Option<(u8, u8, u8)> {
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            return Some((
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            ));
        }
        return None;
    }
    let parts: Vec<&str> = s.split(',').map(|p| p.trim()).collect();
    if parts.len() == 3 {
        return Some((
            parts[0].parse().ok()?,
            parts[1].parse().ok()?,
            parts[2].parse().ok()?,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_hex_and_triplet_parse() {
        assert_eq!(parse_rgb("orange"), Some((255, 140, 0)));
        assert_eq!(parse_rgb("Purple"), Some((180, 100, 255)));
        assert_eq!(parse_rgb("#1affc0"), Some((0x1a, 0xff, 0xc0)));
        assert_eq!(parse_rgb("10, 20, 30"), Some((10, 20, 30)));
    }

    #[test]
    fn empty_or_invalid_falls_back_to_default() {
        assert_eq!(parse_rgb(""), None);
        assert_eq!(parse_rgb("not-a-color"), None);
        assert_eq!(parse_rgb("#zzz"), None);
        // set_accent must never leave the accent unreadable.
        set_accent("garbage");
        assert_eq!(accent(), Color::Rgb(255, 140, 0));
    }
}
