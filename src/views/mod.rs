mod about;
mod carriers;
mod construction;
mod engineers;
mod event;
mod explorer;
mod factions;
mod inventory;
mod modules;
mod news;
mod pilot;
mod settings;
mod stations;
mod suit;
mod system;
mod todo_view;
mod trade_routes;
mod util;

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

pub use about::AboutView;
pub use carriers::CarriersView;
pub use construction::ConstructionView;
pub use engineers::EngineersView;
pub use event::ViewEvent;
pub use explorer::ExplorerView;
pub use factions::FactionsView;
pub use inventory::InventoryView;
pub use modules::ModulesView;
pub use news::NewsView;
pub use pilot::PilotView;
pub use settings::SettingsView;
pub use stations::StationsView;
pub use suit::SuitView;
pub use system::SystemView;
pub use todo_view::TodoView;
pub use trade_routes::TradeRoutesView;
