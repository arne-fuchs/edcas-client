pub enum ViewEvent {
    None,
    Consumed,
    Quit,
    NextTab,
    PrevTab,
    SettingsChanged,
    OpenFactions(String),
}
