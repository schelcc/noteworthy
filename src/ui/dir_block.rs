/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::{fs, path::Path, sync::Arc};

use crate::{
    fs_interface::MetadataType,
    intern_error::{self},
};

use super::{super::config, block::FSListBlock, file_item::FileItem};

pub struct DirBlock {
    name: String,
    parent: Box<Path>,
    cursor_idx: usize,
    pub focused: bool,
    content: Vec<FileItem>,
    last_path: Box<Path>,
    selected_content: Vec<FileItem>,
}

impl FSListBlock for DirBlock {
    fn new(title: &'static str, _: Option<Arc<rusqlite::Connection>>) -> Self {
        DirBlock {
            name: String::from(title),
            parent: Path::new("/home/").into(),
            focused: false,
            cursor_idx: 0,
            content: Vec::new(),
            last_path: Path::new("/home/").into(),
            selected_content: Vec::new(),
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_focus(&self) -> bool {
        self.focused
    }

    fn get_parent(&self) -> FileItem {
        FileItem::new().path(self.parent.clone())
    }

    fn get_resolved_content(&self) -> &Vec<FileItem> {
        &self.content
    }

    fn get_resolved_content_mut(&mut self) -> &mut Vec<FileItem> {
        self.content.as_mut()
    }

    fn get_cursor_idx(&self) -> usize {
        self.cursor_idx
    }

    fn set_cursor_idx(&mut self, new_idx: usize) -> () {
        self.cursor_idx = new_idx
    }

    // Probably doesn't need to be a result
    fn set_parent(&mut self, new_parent_file: FileItem) -> Result<(), intern_error::Error> {
        self.parent = new_parent_file.clone().path;
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
        self.content.get_mut(self.cursor_idx)
    }

    fn resolve(&mut self) -> Result<(), intern_error::Error> {
        self.content = Vec::new();

        let paths = fs::read_dir(&self.parent)?;

        if self.parent != Path::new("/").into() {
            self.content
                .push(FileItem::new().file_type(MetadataType::ReturnType).path({
                    let path = &mut self.parent.components();

                    path.next_back();

                    path.as_path().into()
                }));
        };

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => {
                    let file_type = MetadataType::from(res_path.file_type().unwrap());

                    if file_type == MetadataType::ErrorType {
                        continue;
                    };

                    // TODO: Fix error handling
                    if res_path.file_name().to_str().unwrap().starts_with('.')
                        && !config::SETTINGS.show_hidden_files
                    {
                        continue;
                    }

                    self.content.push(
                        FileItem::new()
                            .name(res_path.file_name().to_str().unwrap())
                            .file_type(MetadataType::from(res_path.file_type().unwrap()))
                            .path(res_path.path().into()), // Fix
                    );
                }
            }
        }

        self.content.sort();

        self.last_path = self.parent.clone();

        Ok(())
    }
}
