use std::path::Path;

use crate::fs_interface::MetadataType;

pub struct FileItem {
    pub name: String,
    pub path: Box<Path>,
    pub file_type: MetadataType,
    pub uuid: String,
}

impl FileItem {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            path: Path::new("/").into(),
            file_type: MetadataType::DefaultType,
            uuid: String::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = String::from(name);
        self
    }

    pub fn path(mut self, path: Box<Path>) -> Self {
        self.path = path;
        self
    }

    pub fn file_type(mut self, file_type: MetadataType) -> Self {
        self.file_type = file_type;
        self
    }

    pub fn uuid(mut self, uuid: String) -> Self {
        self.uuid = uuid;
        self
    }
}
