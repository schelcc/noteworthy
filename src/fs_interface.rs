use glob::GlobError;
use ::glob::{self, glob};
use rusqlite::{self, Connection, Error, Result};
use std::{fs, path::{Path, PathBuf}, str::Lines};

#[derive(Debug)]
enum MetadataType {
    CollectionType,
    DocumentType,
    ErrorType
}

#[derive(Debug)]
struct Metadata {
    uuid : String,
    last_modified : String,
    parent : String,
    pinned : bool,
    object_type : MetadataType
}


impl MetadataType {
    fn from_str(s: &str) -> MetadataType {
        match s {
            "CollectionType" => MetadataType::CollectionType,
            "DocumentType" => MetadataType::DocumentType,
            _ => MetadataType::ErrorType
        }
    }
}

impl Metadata {
    fn from_file(path: Result<PathBuf, GlobError>) -> Metadata {
        let file_name = String::from(path.as_ref().unwrap().file_name().unwrap().to_str().unwrap().split(".").nth(0).unwrap());
        
        let file = fs::read_to_string(path.unwrap()).expect("foo");

        let mut row = Metadata { uuid: file_name, last_modified: String::from(""), parent: String::from(""), pinned: false, object_type: MetadataType::ErrorType };

        for line in file.lines() {
            let trim_line = line.trim();
            let kv_pair = trim_line.replace("\"", "");

            let (key, value) = (kv_pair.split(":").nth(0), kv_pair.split(":").nth(1));

            match key {
                None => (),
                Some(res) => match res {
                    "lastModified" => { row.last_modified = value.unwrap().to_string() },
                    "parent" => { row.parent = value.unwrap().to_string().trim().replace(",", "") },
                    "pinned" => { row.pinned = match value.unwrap().parse::<bool>() {
                        Err(_) => false,
                        Ok(res) => res
                    }},
                    "type" => {row.object_type = MetadataType::from_str(value.unwrap()
                        .trim()
                        .replace("\"", "")
                        .replace(",", "").as_ref()) }
                    _ => ()
                }
            }
        };

        row
    }
}

pub fn resolve_file_tree() -> Result<()> {
    let db = Connection::open_in_memory()?;

    db.execute("CREATE TABLE objects (
        uuid TEXT,
        last_modified TEXT,
        parent TEXT,
        pinned NUMBER,
        object_type TEXT)", ())?;

    // let path = "raw-files";
    let path_map;

    match glob("./raw-files/*.metadata") {
        Err(why) => println!("[ERR] Failed to read glob pattern ({:?})", why),
        Ok(paths) => {
            path_map = paths.map(|f| Metadata::from_file(f));
            for item in path_map {
                println!("{:?}", item)
            }
        }
    };

    Ok(())
}