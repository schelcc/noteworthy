#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[ERR] Crossterm : {0}")]
    CrosstermError(String),
    #[error("[ERR] SQLite : {0}")]
    SQLiteError(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Error {
        Error::CrosstermError(value.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Error {
        Error::SQLiteError(value.to_string())
    }
}