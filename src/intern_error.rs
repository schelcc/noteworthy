#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[ERR] Crossterm : {0}")]
    CrosstermError(String),
    #[error("[ERR] SQLite : {0}")]
    SQLiteError(String),
    #[error("[ERR] Internal : Failed to retrieve index {0} from struct")]
    OutOfBoundsError(usize),
    #[error("[ERR] Internal : Cannot walk back any more")]
    WalkBackError,
    #[error("[ERR] Internal : Color could not be converted to RGB")]
    HexToRGBError,
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

// impl Error {
//     fn to_string(self) -> String {
//         self.
//     }
// }
