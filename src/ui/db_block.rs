/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use std::{cmp::Ordering, path::Path, sync::Arc};

use rusqlite::{named_params, Connection};

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

    fn generate_list(
        &self,
        render_area: Rect,
    ) -> Result<Vec<ListItem<'static>>, intern_error::Error> {
        let mut result = Vec::new();

        let offset: Option<usize> = match self.cursor_idx.cmp(&render_area.height.into()) {
            Ordering::Greater => Some(self.cursor_idx - usize::from(render_area.height)),
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
        if self.focused {
            self.resolve()?;
        };

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
                self.parent = selected_file.uuid.clone();
                self.cursor_idx = 0;
            }
            _ => (),
        };

        Ok(())
    }

    fn toggle_highlight_selection(&mut self) -> Result<(), Error> {
        match self.content.get_mut(self.cursor_idx) {
            Some(val) => {
                if !val.highlighted {
                    val.highlighted = true;
                    self.selected_content
                        .push(self.content.get(self.cursor_idx).unwrap().clone());
                } else {
                    // val.highlighted = false;
                    // match self.selected_content.binary_search(val) {
                    //     Err(_) => (),
                    //     Ok(idx) => {
                    //         self.selected_content.remove(idx);
                    //     }
                    // }
                }
                Ok(())
            }
            None => Err(Error::VecAccessError(self.cursor_idx)),
        }
    }
}
