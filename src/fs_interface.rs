use ::glob::{self, glob};
use glob::GlobError;
use rusqlite::{self, named_params, Connection, Error, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    str::Lines,
};

#[derive(Debug)]
enum MetadataType {
    CollectionType,
    DocumentType,
    ErrorType,
}

#[derive(Debug)]
struct Metadata {
    uuid: String,
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

        let file = fs::read_to_string(path.unwrap()).expect("foo");

        let mut row = Metadata {
            uuid: file_name,
            last_modified: String::from(""),
            parent: String::from(""),
            pinned: false,
            object_type: MetadataType::ErrorType,
        };

        for line in file.lines() {
            let trim_line = line.trim();
            let kv_pair = trim_line.replace("\"", "");

            let (key, value) = (kv_pair.split(":").nth(0), kv_pair.split(":").nth(1));

            match key {
                None => (),
                Some(res) => match res {
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
                    _ => (),
                },
            }
        }

        row
    }
}

pub fn resolve_file_tree(db : &Connection) -> Result<(), crate::intern_error::Error> {
    db.execute(
        "CREATE TABLE objects (
        uuid TEXT,
        last_modified TEXT,
        parent TEXT,
        pinned NUMBER,
        object_type TEXT )",
        (),
    )?;

    let mut stmt = db.prepare(
        "INSERT INTO objects VALUES (:uuid, :last_modified, :parent, :pinned, :object_type)",
    )?;

    match glob("./raw-files/*.metadata") {
        Err(why) => println!("[ERR] Failed to read glob pattern ({:?})", why),
        Ok(paths) => {
            paths
                .map(|f| Metadata::from_file(f))
                .map(|f| {
                    stmt.execute(named_params! {
                        ":uuid" : f.uuid,
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
