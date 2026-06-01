mod about;
mod carriers;
mod commander;
mod construction;
pub(super) mod engineers;
mod event;
mod explorer;
mod factions;
pub(super) mod inventory;
pub(super) mod modules;
mod news;
pub(super) mod pilot;
mod search_nearest;
mod settings;
mod stations;
mod todo_view;
mod trade_routes;
mod util;
mod workshop;

/// Appends BGS timing hints to known faction state names.
///
/// - Pending Expansion: faction queued to expand, ~5 days until active
/// - Active Expansion: faction moving to new system, ~3-7 days to complete
pub(super) fn annotate_faction_state(state: &str, is_pending: bool) -> String {
    match (state, is_pending) {
        ("Expansion", true) => "Expansion (~5d until active)".to_string(),
        ("Expansion", false) => "Expansion (~3-7d to complete)".to_string(),
        _ => state.to_string(),
    }
}

pub use carriers::CarriersView;
pub use commander::CommanderView;
pub use construction::ConstructionView;
pub use event::ViewEvent;
pub use explorer::ExplorerView;
pub use factions::FactionsView;
pub use news::NewsView;
pub use search_nearest::SearchNearestView;
pub use settings::SettingsView;
pub use stations::StationsView;
pub use todo_view::TodoView;
pub use trade_routes::TradeRoutesView;
pub use workshop::WorkshopView;
