/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::{path::Path, sync::Arc};

use rusqlite::{named_params, Connection};

use crate::{
    fs_interface::MetadataType,
    intern_error::{self},
};

use super::{block::FSListBlock, file_item::FileItem};

pub struct DBBlock {
    name: String,
    parent: String,
    cursor_idx: usize,
    pub focused: bool,
    content: Vec<FileItem>,
    db_connection: Option<Arc<Connection>>,
    selected_content: Vec<FileItem>,
}

impl FSListBlock for DBBlock {
    // Figure out what the fuck I'm trying to do with lifetimes
    fn new(title: &'static str, db_conn: Option<Arc<Connection>>) -> Self {
        DBBlock {
            name: String::from(title),
            parent: String::from("root"),
            focused: false,
            cursor_idx: 0,
            content: Vec::new(),
            db_connection: db_conn,
            selected_content: Vec::new(),
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_focus(&self) -> bool {
        self.focused
    }

    fn get_resolved_content(&self) -> &Vec<FileItem> {
        &self.content
    }

    fn get_resolved_content_mut(&mut self) -> &mut Vec<FileItem> {
        &mut self.content
    }

    fn get_cursor_idx(&self) -> usize {
        self.cursor_idx
    }

    fn set_cursor_idx(&mut self, new_idx: usize) -> () {
        self.cursor_idx = new_idx
    }

    fn get_parent(&self) -> FileItem {
        FileItem::new().uuid(self.parent.clone())
    }

    // Probably doesn't need to be a result
    fn set_parent(&mut self, new_parent_file: FileItem) -> Result<(), intern_error::Error> {
        self.parent = new_parent_file.clone().uuid;
        Ok(())
    }

    fn get_selected_content(&self) -> &Vec<FileItem> {
        &self.selected_content
    }

    fn get_selected_content_mut(&mut self) -> &mut Vec<FileItem> {
        &mut self.selected_content
    }

    fn clear_selected_content(&mut self) -> () {
        self.selected_content = Vec::new();
    }

    fn get_cursor_selection_mut(&mut self) -> Option<&mut FileItem> {
        self.selected_content.get_mut(self.cursor_idx)
    }

    fn resolve(&mut self) -> Result<(), crate::intern_error::Error> {
        self.content = Vec::new();

        // Check for none case of db initialization
        match self.db_connection {
            Some(_) => (),
            None => return Err(intern_error::Error::NoDBConnectionError),
        };

        // If we pass the above check we have db, so we can safely create a local reference unwrapped
        let db = self.db_connection.as_ref().unwrap();

        let mut stmt = db.prepare("SELECT uuid, name, object_type FROM objects WHERE parent=?1")?;

        let file_iter = stmt.query_map([&self.parent], |r| {
            Ok(FileItem {
                uuid: r.get(0)?,
                path: Path::new(".").into(), // Maybe make the path field an option in the future
                name: r.get(1)?,
                file_type: MetadataType::from(r.get::<usize, String>(2)?),
                highlighted: false,
            })
        })?;

        match db.query_row(
            "SELECT parent FROM objects WHERE uuid=:uuid",
            named_params! {":uuid" : self.parent},
            |r| Ok((r.get::<usize, String>(0), r.get::<usize, String>(1))),
        ) {
            Err(_) => (),
            Ok(val) => {
                self.content.push(FileItem {
                    name: String::new(),
                    path: Path::new(".").into(),
                    file_type: MetadataType::ReturnType,
                    uuid: val.0?,
                    highlighted: false,
                });
            }
        };

        for row in file_iter {
            if let Ok(row) = row {
                self.content.push(row);
            }
        }

        self.content.sort();

        Ok(())
    }
}
