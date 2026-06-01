pub enum ViewEvent {
    None,
    Consumed,
    SettingsChanged,
    OpenFactions(String),
    OpenSearchNearest { commodity: String, canonical_name: String, system: String, ship_pad_size: char },
    TrackConstruction(i64),
}
