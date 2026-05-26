pub enum ViewEvent {
    None,
    Consumed,
    SettingsChanged,
    OpenFactions(String),
    OpenSearchNearest { commodity: String, system: String },
}
