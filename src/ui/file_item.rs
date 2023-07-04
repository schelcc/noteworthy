/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::path::Path;

use crate::fs_interface::MetadataType;

#[derive(Clone)]
pub struct FileItem {
    pub name: String,
    pub path: Box<Path>,
    pub file_type: MetadataType,
    pub uuid: String,
    pub highlighted: bool,
}

// ** For sorting vecs of file items **
impl Ord for FileItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let type_ordering = self.file_type.cmp(&other.file_type);

        if type_ordering == std::cmp::Ordering::Equal {
            self.name.to_lowercase().cmp(&other.name.to_lowercase())
        } else {
            type_ordering
        }
    }
}

impl PartialOrd for FileItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FileItem {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for FileItem {}

impl FileItem {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            path: Path::new("/").into(),
            file_type: MetadataType::DefaultType,
            uuid: String::new(),
            highlighted: false,
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
