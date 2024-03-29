/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use glob::PatternError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("[!ERR!] ! Placeholder error !")]
    PlaceholderError,
    #[error("[ERR] Crossterm : {0}")]
    CrosstermError(String),
    #[error("[ERR] SQLite : {0}")]
    SQLiteError(String),
    #[error("[ERR] Internal : Failed to retrieve index {0} from struct")]
    OutOfBoundsError(usize),
    #[error("[ERR] Internal : Cannot walk back any more")]
    WalkBackError,
    #[error("[ERR] Internal : Color could not be converted to RGB")]
    HexToRGBError(String),
    #[error("[ERR] Configparser : {0}")]
    ConfigparserError(String),
    #[error("[ERR] SSH2 : {0}")]
    SSHError(String),
    #[error("[ERR] Internal : DB Connection not initialized, cannot populate DBBlock")]
    NoDBConnectionError,
    #[error("[ERR] Internal : Couldn't access item at index {0}")]
    VecAccessError(usize),
    #[error("[ERR] Internal : Couldn't remove selected item at index {0}")]
    VecRemoveError(usize),
    #[error("[ERR] Internal : Glob error")]
    GlobErr,
    #[error("[ERR] Internal : Save data read failure")]
    JSONParseErr,
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

impl From<ssh2::Error> for Error {
    fn from(value: ssh2::Error) -> Self {
        Error::SSHError(value.to_string())
    }
}

impl From<PatternError> for Error {
    fn from(_value: PatternError) -> Self {
        Self::GlobErr
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JSONParseErr
    }
}

// impl Error {
//     fn to_string(self) -> String {
//         self.
//     }
// }
