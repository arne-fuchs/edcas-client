use postgres::Error;

#[derive(Debug)]
pub struct EdcasError(pub String);

impl std::fmt::Display for EdcasError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EdcasError {}

impl From<Error> for EdcasError {
    fn from(err: Error) -> Self {
        EdcasError(err.to_string())
    }
}

impl From<String> for EdcasError {
    fn from(err: String) -> Self {
        EdcasError(err)
    }
}
