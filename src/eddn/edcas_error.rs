#[derive(Debug)]
pub struct EdcasError(pub ErrorType, pub Option<i64>);

#[derive(Debug)]
pub enum ErrorType {
    GeneralError(String),
    Unimplemented(),
    UnknownEvent(String),
    DbError(postgres::Error),
}
impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ErrorType::GeneralError(error) => write!(f, "{}", error),
            ErrorType::DbError(error) => write!(f, "{}", error),
            ErrorType::Unimplemented() => write!(f, "Unimplemented"),
            ErrorType::UnknownEvent(event) => write!(f, "UnknownEvent: {}", event),
        }
    }
}
impl std::fmt::Display for EdcasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{:>15}]:{}",
            match self.1 {
                Some(id) => id.to_string(),
                None => {
                    "None".into()
                }
            },
            self.0
        )
    }
}

impl std::error::Error for EdcasError {}

impl From<postgres::Error> for EdcasError {
    fn from(err: postgres::Error) -> Self {
        EdcasError(ErrorType::DbError(err), None)
    }
}

impl From<String> for EdcasError {
    fn from(err: String) -> Self {
        EdcasError(ErrorType::GeneralError(err), None)
    }
}
impl EdcasError {
    pub fn new(err: impl Into<String>) -> Self {
        EdcasError(ErrorType::GeneralError(err.into()), None)
    }
    pub fn unimplemented() -> Self {
        EdcasError(ErrorType::Unimplemented(), None)
    }
    pub fn unknown_event(event: String) -> Self {
        EdcasError(ErrorType::UnknownEvent(event), None)
    }
}

impl EdcasError {
    pub fn with_id(mut self, journal_id: i64) -> EdcasError {
        self.1 = Some(journal_id);
        self
    }
}
