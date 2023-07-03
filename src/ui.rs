/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

pub mod block;
pub mod db_block;
pub mod dir_block;
pub mod file_item;

use std::sync::Arc;

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    Frame,
};

use crate::intern_error;

use self::{block::FSListBlock, db_block::DBBlock, dir_block::DirBlock};

#[derive(PartialEq)]
enum FileUIFocus {
    Local,
    Remote,
}

pub enum CursorDirection {
    Up,
    Down,
    PgUp,
    PgDn,
}

pub struct FileUI {
    local: DirBlock,
    remote: DBBlock,
    focus: FileUIFocus,
}

const WIDGET_OFFSET: u16 = 3;

impl FileUI {
    // Create the layout and then render generated widgets
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) -> Result<(), intern_error::Error> {
        let layout = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        self.local.focused = self.focus == FileUIFocus::Local;
        self.remote.focused = self.focus == FileUIFocus::Remote;

        f.render_widget(self.local.render(layout[0])?, layout[0]);
        f.render_widget(self.remote.render(layout[1])?, layout[1]);

        Ok(())
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FileUIFocus::Local => FileUIFocus::Remote,
            FileUIFocus::Remote => FileUIFocus::Local,
        };
    }

    pub fn cursor_move(&mut self, direction: CursorDirection) {
        match self.focus {
            FileUIFocus::Local => self.local.cursor_move(direction),
            FileUIFocus::Remote => self.remote.cursor_move(direction),
        };

        self.local.focused = self.focus == FileUIFocus::Local;
        self.remote.focused = self.focus == FileUIFocus::Remote;
    }

    pub fn expand_selection(&mut self) -> Result<(), intern_error::Error> {
        match self.focus {
            FileUIFocus::Local => self.local.expand_selection(),
            FileUIFocus::Remote => self.remote.expand_selection(),
        };

        Ok(())
    }
}

// Helper function to give a FileUI struct
pub fn file_ui(db: Arc<rusqlite::Connection>) -> Result<FileUI, crate::intern_error::Error> {
    let mut ui = FileUI {
        local: DirBlock::new("dir", None),
        remote: DBBlock::new("db", Some(db)),
        focus: FileUIFocus::Local,
    };

    ui.local.resolve()?;
    ui.remote.resolve()?;

    Ok(ui)
}
