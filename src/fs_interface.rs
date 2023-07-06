/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use ::glob::{self, glob};
use glob::GlobError;
use rusqlite::{self, named_params, types::FromSql, Connection, Result};
use std::{fs, path::PathBuf, sync::Arc};

// use crate::intern_error;

#[derive(Copy, Debug, Default, Clone)]
pub enum MetadataType {
    ReturnType,
    CollectionType,
    DocumentType,
    ErrorType,
    #[default]
    DefaultType,
}

impl Ord for MetadataType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for MetadataType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for MetadataType {
    fn eq(&self, other: &Self) -> bool {
        (*self as u8).cmp(&(*other as u8)) == std::cmp::Ordering::Equal
    }
}

impl Eq for MetadataType {}

impl FromSql for MetadataType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        String::column_result(value).map(|f| match f.as_str() {
            "CollectionType" => Self::CollectionType,
            "DocumentType" => Self::DocumentType,
            _ => Self::ErrorType,
        })
    }
}

impl From<std::fs::FileType> for MetadataType {
    fn from(value: std::fs::FileType) -> Self {
        if value.is_dir() {
            Self::CollectionType
        } else if value.is_file() {
            Self::DocumentType
        } else {
            Self::ErrorType
        }
    }
}

impl From<()> for MetadataType {
    fn from(_value: ()) -> Self {
        MetadataType::ErrorType
    }
}

impl From<String> for MetadataType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "CollectionType" => Self::CollectionType,
            "DocumentType" => Self::DocumentType,
            _ => Self::ErrorType,
        }
    }
}

#[derive(Debug, Default)]
struct Metadata {
    uuid: String,
    name: String,
    last_modified: String,
    parent: String,
    pinned: bool,
    object_type: MetadataType,
}

impl MetadataType {
    fn from_str(s: &str) -> MetadataType {
        match s {
            "CollectionType" => MetadataType::CollectionType,
            "DocumentType" => MetadataType::DocumentType,
            _ => MetadataType::ErrorType,
        }
    }

    fn as_str(&self) -> &str {
        match &self {
            Self::DocumentType => "DocumentType",
            Self::CollectionType => "CollectionType",
            Self::ErrorType => "ErrorType",
            Self::DefaultType => "DefaultType",
            Self::ReturnType => "ReturnType",
        }
    }
}

impl Metadata {
    fn from_file(path: Result<PathBuf, GlobError>) -> Metadata {
        let file_name = String::from(
            path.as_ref()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .split('.')
                .next()
                .unwrap(),
        );
        //fix
        let file = fs::read_to_string(path.unwrap()).expect("foo");

        let mut row = Metadata {
            uuid: file_name,
            ..Default::default()
        };

        for line in file.lines() {
            let trim_line = line.trim();
            let kv_pair = trim_line.replace('\"', "");

            let (key, value) = (kv_pair.split(':').next(), kv_pair.split(':').nth(1));

            match key {
                None => (),
                Some(res) => match res {
                    // TODO: Clean up .unwrap() usage
                    "lastModified" => {
                        row.last_modified = value.unwrap().trim().to_string().replace(',', "")
                    }
                    "parent" => {
                        if value.as_ref().unwrap().to_string().trim().replace(',', "") == "" {
                            row.parent = String::from("root");
                        } else {
                            row.parent = value.unwrap().to_string().trim().replace(',', "");
                        }
                    }
                    "pinned" => row.pinned = value.unwrap().parse::<bool>().unwrap_or(false),
                    "type" => {
                        row.object_type = MetadataType::from_str(
                            value.unwrap().trim().replace(['\"', ','], "").as_ref(),
                        )
                    }
                    "visibleName" => {
                        row.name = value.unwrap().to_string().trim().replace(',', "");
                    }
                    _ => (),
                },
            }
        }

        row
    }
}

pub fn resolve_file_tree(db: Arc<Connection>) -> Result<(), crate::intern_error::Error> {
    db.execute(
        "CREATE TABLE objects (
        uuid TEXT,
        name TEXT,
        last_modified TEXT,
        parent TEXT,
        pinned NUMBER,
        object_type TEXT )",
        (),
    )?;

    let mut stmt = db.prepare(
        "INSERT INTO objects VALUES (:uuid, :name, :last_modified, :parent, :pinned, :object_type)",
    )?;

    match glob("./raw-files/*.metadata") {
        Err(why) => println!("[ERR] Failed to read glob pattern ({:?})", why),
        Ok(paths) => {
            paths
                .map(Metadata::from_file)
                .map(|f| {
                    stmt.execute(named_params! {
                        ":uuid" : f.uuid,
                        ":name" : f.name,
                        ":last_modified" : f.last_modified,
                        ":parent" : f.parent,
                        ":pinned" : f.pinned,
                        ":object_type" : f.object_type.as_str(),
                    })
                })
                .for_each(drop);
        }
    };

    Ok(())
}
