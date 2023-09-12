/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use rusqlite::Connection;
use std::{cmp::Ordering, sync::Arc};
use tui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::{
    fs_interface::MetadataType,
    intern_error::{self, Error},
};

use super::{super::config, file_item::FileItem, CursorDirection};

pub trait FSListBlock {
    fn new(title: &'static str, db_conn: Option<Arc<Connection>>) -> Self
    where
        Self: Sized;

    fn get_name(&self) -> String;
    fn get_focus(&self) -> bool;
    fn get_parent(&self) -> FileItem;

    fn get_offset_pos(&self) -> usize;
    fn set_offset_pos(&mut self, new_pos: usize);

    fn get_render_area(&self) -> Rect;
    fn set_render_area(&mut self, area: Rect);

    fn get_resolved_content(&self) -> &Vec<FileItem>;
    fn get_resolved_content_mut(&mut self) -> &mut Vec<FileItem>;

    fn get_selected_content(&self) -> &Vec<FileItem>;
    fn get_selected_content_mut(&mut self) -> &mut Vec<FileItem>;
    fn clear_selected_content(&mut self);

    fn get_cursor_idx(&self) -> usize;
    fn set_cursor_idx(&mut self, new_idx: usize);

    fn set_parent(&mut self, new_parent: FileItem) -> Result<(), intern_error::Error>;

    fn get_cursor_selection(&self) -> Option<&FileItem> {
        self.get_resolved_content().get(self.get_cursor_idx())
    }

    // Must be implemented on child side due to mutability
    // Does it??
    fn get_cursor_selection_mut(&mut self) -> Option<&mut FileItem>;

    fn add_selected_content(&mut self, new_item: FileItem) {
        self.get_selected_content_mut().push(new_item);
    }

    fn remove_selected_content(
        &mut self,
        target_item: FileItem,
    ) -> Result<(), intern_error::Error> {
        match self.get_selected_content_mut().binary_search(&target_item) {
            Err(_) => Err(Error::VecRemoveError(self.get_cursor_idx())),
            Ok(idx) => {
                self.get_selected_content_mut().remove(idx);
                Ok(())
            }
        }
    }

    fn resolve(&mut self) -> Result<(), intern_error::Error>;

    fn generate_list(&self, render_area: Rect) -> Result<Vec<ListItem>, intern_error::Error> {
        let mut result: Vec<ListItem> = Vec::new();

        let adj_height = usize::from(render_area.height - super::WIDGET_OFFSET);

        for (idx, item) in self.get_resolved_content().iter().enumerate() {
            if idx < self.get_offset_pos() || idx > self.get_offset_pos() + adj_height {
                continue;
            }

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

            if self.get_focus() && idx.cmp(&self.get_cursor_idx()) == Ordering::Equal {
                style = style.add_modifier(Modifier::REVERSED)
            };

            result.push(ListItem::new(file_name).style(style))
        }

        Ok(result)
    }

    fn render(&mut self, render_area: Rect) -> Result<List, Error> {
        self.set_render_area(render_area);

        Ok(List::new(self.generate_list(render_area).unwrap())
            .block(
                Block::default()
                    .title(self.get_name())
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double),
            )
            .style(
                Style::default()
                    .fg(config::THEME.foreground)
                    .bg(config::THEME.background),
            ))
    }

    fn cursor_move(&mut self, direction: CursorDirection) {
        let delta: isize = match direction {
            CursorDirection::Down => 1,
            CursorDirection::Up => -1,
            CursorDirection::PgDn => 15,
            CursorDirection::PgUp => -15,
        };

        let adj_height = usize::from(self.get_render_area().height - super::WIDGET_OFFSET);

        let new_pos = if let Some(val) = self.get_cursor_idx().checked_add_signed(delta) {
            if val > self.get_resolved_content().len() - 1 {
                self.get_resolved_content().len() - 1
            } else {
                val
            }
        } else {
            0
        };

        let (clamp_min, clamp_max) = (self.get_offset_pos(), self.get_offset_pos() + adj_height);

        if new_pos <= clamp_min {
            self.set_offset_pos(new_pos)
        } else if new_pos >= clamp_max {
            self.set_offset_pos(new_pos - adj_height)
        }

        self.set_cursor_idx(new_pos);
    }

    fn expand_selection(&mut self) -> Result<(), Error> {
        let prev_parent = self.get_parent();

        let selected_file = match self.get_cursor_selection() {
            Some(val) => val.clone(),
            None => return Err(Error::VecAccessError(self.get_cursor_idx())),
        };

        match selected_file.file_type {
            MetadataType::CollectionType | MetadataType::ReturnType => {
                self.set_parent(selected_file)?;
                self.set_cursor_idx(0);
            }
            _ => (),
        };

        match self.resolve() {
            Err(why) => {
                self.set_parent(prev_parent)?;
                // We can safely ignore this result because if the user
                // asked to expand something then we know the parent was valid
                let _ = self.resolve();
                Err(why)
            }
            Ok(_) => Ok(()),
        }
    }

    fn refresh_view(&mut self) -> Result<(), Error> {
        self.resolve()?;

        Ok(())
    }

    fn toggle_highlight_selection(&mut self) -> Result<(), Error> {
        let current_selection = { self.get_cursor_selection().unwrap().clone() };
        // let idx = { self.get_cursor_idx() };
        match self.get_cursor_selection_mut() {
            Some(val) => {
                if !val.highlighted {
                    val.highlighted = true;
                    self.add_selected_content(current_selection);
                } else {
                    val.highlighted = false;
                    // Unsure if a result is needed for removal
                    let _ = self.remove_selected_content(current_selection);
                }
                Ok(())
            }
            None => Err(intern_error::Error::VecAccessError(self.get_cursor_idx())),
        }
    }
}
