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
