/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::{cmp::Ordering, fs, path::Path, sync::Arc};

use tui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::{
    fs_interface::MetadataType,
    intern_error::{self, Error},
};

use super::{super::config, block::FSListBlock, file_item::FileItem, CursorDirection};

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

    fn generate_list(
        &self,
        render_area: Rect,
    ) -> Result<Vec<ListItem<'static>>, intern_error::Error> {
        let mut result = Vec::new();

        let adj_height = render_area.height - super::WIDGET_OFFSET;

        let offset: Option<usize> = match self.cursor_idx.cmp(&adj_height.into()) {
            Ordering::Greater => Some(self.cursor_idx - usize::from(adj_height)),
            _ => None,
        };

        for (idx, item) in self.content.iter().enumerate() {
            match offset {
                Some(val) => {
                    if idx < val || idx > self.cursor_idx {
                        continue;
                    }
                }
                None => (),
            };

            let mut file_name = match item.file_type {
                MetadataType::CollectionType => String::from("/"),
                MetadataType::ReturnType => String::from("../"),
                _ => String::from(" "),
            };

            let mut style = match item.file_type {
                MetadataType::CollectionType | MetadataType::ReturnType => {
                    Style::default().add_modifier(Modifier::BOLD)
                }
                MetadataType::DocumentType => Style::default().add_modifier(Modifier::ITALIC),
                MetadataType::DefaultType | MetadataType::ErrorType => Style::default(),
            };

            if item.highlighted {
                style = style.fg(config::THEME.highlight)
            };

            if item.file_type == MetadataType::CollectionType
                || item.file_type == MetadataType::DocumentType
            {
                file_name.push_str(&item.name);
            };

            if self.focused && idx.cmp(&self.cursor_idx) == Ordering::Equal {
                style = style.add_modifier(Modifier::REVERSED)
            };

            result.push(ListItem::new(file_name).style(style));
        }

        Ok(result)
    }

    fn render(&mut self, render_area: Rect) -> Result<List, Error> {
        Ok(List::new(self.generate_list(render_area).unwrap())
            .block(
                Block::default()
                    .title(self.name.clone())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double),
            )
            .style(
                Style::default()
                    .fg(config::THEME.foreground)
                    .bg(config::THEME.background),
            ))
    }

    fn cursor_move(&mut self, direction: super::CursorDirection) {
        let delta: isize = match direction {
            CursorDirection::Down => 1,
            CursorDirection::Up => -1,
            CursorDirection::PgDn => 15,
            CursorDirection::PgUp => -15,
        };

        self.cursor_idx = match self.cursor_idx.checked_add_signed(delta) {
            Some(val) => match val.cmp(&self.content.len()) {
                Ordering::Less => val,
                _ => self.cursor_idx,
            },
            None => self.cursor_idx,
        };
    }

    fn expand_selection(&mut self) -> Result<(), Error> {
        let selected_file = match self.content.get(self.cursor_idx) {
            Some(val) => val,
            None => return Err(Error::VecAccessError(self.cursor_idx)),
        };

        match selected_file.file_type {
            MetadataType::CollectionType | MetadataType::ReturnType => {
                self.parent = selected_file.path.clone().into();
                self.cursor_idx = 0;
            }
            _ => (),
        };

        match self.resolve() {
            Err(why) => {
                self.parent = self.last_path.clone();
                self.resolve()?;
                Err(why)
            }
            Ok(_) => Ok(()),
        }
    }

    fn toggle_highlight_selection(&mut self) -> Result<(), Error> {
        match self.content.get_mut(self.cursor_idx) {
            Some(val) => {
                val.highlighted = true;
                self.selected_content
                    .push(self.content.get(self.cursor_idx).unwrap().clone());
                Ok(())
            }
            None => Err(Error::VecAccessError(self.cursor_idx)),
        }
    }
}
