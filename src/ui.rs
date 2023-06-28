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

pub struct FileUI<'a> {
    local: DirBlock,
    remote: DBBlock<'a>,
    focus: FileUIFocus,
}

const WIDGET_OFFSET: usize = 3;

impl FileUI<'_> {
    // Create the layout and then render generated widgets
    pub fn render<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        db: &Connection,
    ) -> Result<(), intern_error::Error> {
        let layout = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        // self.local.focused = self.focus == FileUIFocus::Local;
        // self.remote.focused = self.focus == FileUIFocus::Remote;

        // // TODO : Change to only repopulate upon changes
        // self.local.resolve_from_dir()?;
        // self.remote.resolve_from_db(db)?;

        // self.local.content.render_height = layout[0].height.into();
        // self.remote.content.render_height = layout[1].height.into();

        // self.local.content.render_height -= WIDGET_OFFSET;
        // self.remote.content.render_height -= WIDGET_OFFSET;

        // f.render_widget(self.local.spawn_widget(), layout[0]);
        // f.render_widget(self.remote.spawn_widget(), layout[1]);

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
        // let focused_block = match self.focus {
        //     FileUIFocus::Local => &mut self.local,
        //     FileUIFocus::Remote => &mut self.remote,
        // };

        // let mut updated = false;

        // focused_block.parent = match focused_block.content.get(focused_block.content.cursor_idx) {
        //     Some(val) => match val.file_type {
        //         MetadataType::CollectionType | MetadataType::ReturnType => {
        //             updated = true;
        //             match self.focus {
        //                 FileUIFocus::Local => (*val.path).display().to_string(),
        //                 FileUIFocus::Remote => (*val.uuid).to_string(),
        //             }
        //         }
        //         _ => focused_block.parent.clone(),
        //     },
        //     None => {
        //         return Err(intern_error::Error::OutOfBoundsError(
        //             focused_block.content.cursor_idx,
        //         ))
        //     }
        // };

        // if updated {
        //     focused_block.content.cursor_idx = 0;
        // }

        Ok(())
    }
}

// Helper function to give a FileUI struct
pub fn file_ui<'b>(db: &'b Connection) -> Result<FileUI, crate::intern_error::Error> {
    let mut ui = FileUI {
        local: DirBlock::new("dir"),
        remote: DBBlock::new("db").set_db_connection(&db),
        focus: FileUIFocus::Local,
    };

    ui.local.resolve()?;
    ui.remote.resolve()?;

    Ok(ui)
}
