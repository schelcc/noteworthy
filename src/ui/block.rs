/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

use rusqlite::Connection;
use std::sync::Arc;
use tui::{
    layout::Rect,
    widgets::{List, ListItem},
};

use crate::intern_error::{self, Error};

use super::CursorDirection;

pub trait FSListBlock {
    fn new(title: &'static str, db_conn: Option<Arc<Connection>>) -> Self
    where
        Self: Sized;

    fn resolve(&mut self) -> Result<(), intern_error::Error>;

    fn generate_list(&self, render_area: Rect) -> Result<Vec<ListItem>, intern_error::Error>;

    fn render(&mut self, render_area: Rect) -> Result<List, Error>;

    fn cursor_move(&mut self, direction: CursorDirection);

    fn expand_selection(&mut self) -> Result<(), Error>;

    fn toggle_highlight_selection(&mut self) -> Result<(), Error>;
}
