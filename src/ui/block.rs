use tui::{
    layout::Rect,
    widgets::{List, ListItem},
};

use crate::intern_error;

use super::{file_item::FileItem, CursorDirection};

pub trait FSListBlock {
    fn new(title: &'static str) -> Self;

    fn resolve(&mut self) -> Result<(), intern_error::Error>;

    fn generate_list(
        &self,
        render_area: Rect,
    ) -> Result<Vec<ListItem<'static>>, intern_error::Error>;

    fn render(&mut self, render_area: Rect) -> List<'static>;

    fn cursor_move(&mut self, direction: CursorDirection);
}
