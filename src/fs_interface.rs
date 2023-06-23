use ::glob::{self, glob};
use glob::GlobError;
use rusqlite::{self, named_params, types::FromSql, Connection, Result};
use std::{fs, path::PathBuf};

use crate::intern_error;

#[derive(Debug, Default, PartialEq)]
pub enum MetadataType {
    CollectionType,
    ReturnType,
    DocumentType,
    ErrorType,
    #[default]
    DefaultType,
}

impl FromSql for MetadataType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        String::column_result(value).and_then(|f| match f.as_str() {
            "CollectionType" => Ok(Self::CollectionType),
            "DocumentType" => Ok(Self::DocumentType),
            _ => Ok(Self::ErrorType),
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

    fn to_str(&self) -> &str {
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
                .split(".")
                .nth(0)
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
            let kv_pair = trim_line.replace("\"", "");

            let (key, value) = (kv_pair.split(":").nth(0), kv_pair.split(":").nth(1));

            match key {
                None => (),
                Some(res) => match res {
                    // TODO: Clean up .unwrap() usage
                    "lastModified" => {
                        row.last_modified = value.unwrap().trim().to_string().replace(",", "")
                    }
                    "parent" => {
                        if value.as_ref().unwrap().to_string().trim().replace(",", "") == "" {
                            row.parent = String::from("root");
                        } else {
                            row.parent = value.unwrap().to_string().trim().replace(",", "");
                        }
                    }
                    "pinned" => {
                        row.pinned = match value.unwrap().parse::<bool>() {
                            Err(_) => false,
                            Ok(res) => res,
                        }
                    }
                    "type" => {
                        row.object_type = MetadataType::from_str(
                            value
                                .unwrap()
                                .trim()
                                .replace("\"", "")
                                .replace(",", "")
                                .as_ref(),
                        )
                    }
                    "visibleName" => {
                        row.name = value.unwrap().to_string().trim().replace(",", "");
                    }
                    _ => (),
                },
            }
        }

        row
    }
}

pub fn resolve_file_tree(db: &Connection) -> Result<(), crate::intern_error::Error> {
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
                .map(|f| Metadata::from_file(f))
                .map(|f| {
                    stmt.execute(named_params! {
                        ":uuid" : f.uuid,
                        ":name" : f.name,
                        ":last_modified" : f.last_modified,
                        ":parent" : f.parent,
                        ":pinned" : f.pinned,
                        ":object_type" : f.object_type.to_str(),
                    })
                })
                .for_each(drop);
        }
    };

    Ok(())
}
