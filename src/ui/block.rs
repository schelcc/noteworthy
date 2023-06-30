use std::sync::Arc;

use rusqlite::Connection;
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

    fn expand_selection(&mut self) -> ();
}
