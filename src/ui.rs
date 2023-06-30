pub mod block;
pub mod db_block;
pub mod dir_block;
pub mod file_item;

use std::{
    cmp::Ordering,
    fs,
    ops::BitAnd,
    os::unix::prelude::PermissionsExt,
    path::{Components, Path},
    sync::Arc,
};

use rusqlite::{named_params, Connection};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    config,
    fs_interface::{self, MetadataType},
    intern_error,
    notification::{NotificationType, NotificationWidget},
};

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

        f.render_widget(self.local.render(layout[0]), layout[0]);
        f.render_widget(self.remote.render(layout[1]), layout[1]);

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
        }

        Ok(())
    }
}

// Helper function to give a FileUI struct
pub fn file_ui(db: Arc<Connection>) -> Result<FileUI, crate::intern_error::Error> {
    let mut ui = FileUI {
        local: DirBlock::new("dir", None),
        remote: DBBlock::new("db", Some(db)),
        focus: FileUIFocus::Local,
    };

    ui.local.resolve()?;
    ui.remote.resolve()?;

    Ok(ui)
}
