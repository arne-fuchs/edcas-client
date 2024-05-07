pub struct About {
    pub(crate) github_url: String,
    pub(crate) discord_url: String
}

impl Default for About {
    fn default() -> Self {
        Self {
            github_url: "https://github.com/arne-fuchs/edcas-client".to_owned(),
            discord_url: "https://discord.gg/fsstTkAw".to_owned()
        }
    }
}
