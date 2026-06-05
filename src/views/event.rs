pub enum ViewEvent {
    None,
    Consumed,
    SettingsChanged,
    OpenFactions(String),
    OpenSearchNearest { commodity: String, canonical_name: String, system: String, ship_pad_size: char },
    OpenMultiSearch { commodities: Vec<String>, system: String, ship_pad_size: char },
    TrackConstruction(i64),
    /// Toggle whether the fleet carrier with this market_id is owned by the commander
    /// (persisted to my_carriers.json and reflected in journal.carrier_cargo).
    ToggleMyCarrier(i64),
}
