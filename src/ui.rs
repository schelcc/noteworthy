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

#[derive(Default)]
pub enum Display {
    #[default]
    LocalRemoteSplit,
}

// #[derive(Default)]
pub struct FileItem {
    uuid: String,
    name: String,
    path: Box<Path>,
    file_type: MetadataType,
}

#[derive(Default)]
pub struct FileList {
    content: Vec<FileItem>,
    cursor_idx: usize,
    render_height: usize,
}

// #[derive(Default)]
struct FSBlock {
    name: String,
    parent: String,
    content: FileList,
    focused: bool,
}

impl Default for FSBlock {
    fn default() -> Self {
        FSBlock {
            name: String::default(),
            parent: String::default(),
            content: FileList::default(),
            focused: false,
        }
    }
}

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
    local: FSBlock,
    remote: FSBlock,
    focus: FileUIFocus,
}

const WIDGET_OFFSET: usize = 3;

impl FileList {
    // Forward .push() for wrapper struct
    fn push(&mut self, value: FileItem) {
        self.content.push(value);
    }

    fn new() -> Self {
        FileList {
            content: Vec::new(),
            cursor_idx: 0,
            ..Default::default()
        }
    }

    // Forward .get() for wrapper struct
    fn get(&self, idx: usize) -> Option<&FileItem> {
        self.content.get(idx)
    }

    // Adjust the currently selected row dependent on the CursorDirection struct
    pub fn cusor_move(&mut self, direction: CursorDirection) {
        let delta = match direction {
            CursorDirection::Up => -1,
            CursorDirection::Down => 1,
            CursorDirection::PgUp => -15,
            CursorDirection::PgDn => 15,
        };

        // Ensure cursor doesn't try to go to a row that doesn't exist
        self.cursor_idx = match self.cursor_idx.checked_add_signed(delta) {
            Some(val) => match &val.cmp(&self.content.len()) {
                Ordering::Less => val,
                Ordering::Equal | Ordering::Greater => self.content.len() - 1,
            },
            None => 0,
        };
    }

    // Convert FileList to Vec<ListItem> for tui to generate an array of ListItems from
    // Applies cursor styling, depends on focus bool, hiding underline if not focused
    pub fn to_item_list(&self, focus: bool) -> Vec<ListItem> {
        let mut result: Vec<ListItem> = Vec::new();

        let offset = match self.cursor_idx.cmp(&self.render_height) {
            Ordering::Greater => Some(self.cursor_idx - self.render_height),
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
            // Add "/" prefix to any CollectionTypes to denote directory
            let mut file_name = match item.file_type {
                MetadataType::CollectionType => String::from("/"),
                MetadataType::ReturnType => String::from("../"),
                _ => String::from(" "),
            };

            let mut style = Style::default();

            match item.file_type {
                MetadataType::CollectionType => {
                    style = style.add_modifier(Modifier::BOLD);
                }
                MetadataType::DocumentType => {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                MetadataType::ReturnType => {
                    style = style.add_modifier(Modifier::BOLD);
                }
                MetadataType::ErrorType | MetadataType::DefaultType => {
                    // Skip error types
                    continue;
                }
            }

            if item.file_type == MetadataType::CollectionType
                || item.file_type == MetadataType::DocumentType
            {
                file_name.push_str(&item.name);
            }

            // Push ListItem w/ underline if focused, otherwise just regular ListItem
            if focus && idx.cmp(&self.cursor_idx) == Ordering::Equal {
                style = style
                    .add_modifier(Modifier::UNDERLINED)
                    .add_modifier(Modifier::REVERSED);
            }

            result.push(ListItem::new(file_name).style(style));
        }

        result
    }
}

impl FSBlock {
    // Populate FileList from a rusqlite DB
    fn resolve_from_db(&mut self, db: &Connection) -> Result<(), intern_error::Error> {
        self.content.content = Vec::new();

        let mut stmt = db.prepare("SELECT uuid, name, object_type FROM objects WHERE parent=?1")?;

        let file_iter = stmt.query_map([&self.parent], |r| {
            Ok(FileItem {
                uuid: r.get(0)?,
                path: Path::new(&r.get::<usize, String>(0)?).into(),
                name: r.get(1)?,
                file_type: MetadataType::from(r.get::<usize, String>(2)?),
            })
        })?;

        match db.query_row(
            "SELECT parent,uuid FROM objects WHERE uuid=:uuid",
            named_params! {":uuid":self.parent},
            |r| Ok((r.get::<usize, String>(0), r.get::<usize, String>(1))),
        ) {
            Err(_) => (),
            Ok(val) => {
                self.content.push(FileItem {
                    name: val.0?,
                    path: Path::new(&val.1?).into(),
                    file_type: MetadataType::ReturnType,
                    uuid: String::from(""),
                });
            }
        };

        for row in file_iter {
            if let Ok(row) = row {
                self.content.push(row);
            }
        }

        Ok(())
    }

    // Populate FileList from a directory
    fn resolve_from_dir(&mut self) -> Result<(), intern_error::Error> {
        self.content.content = Vec::new();

        let paths = fs::read_dir(&self.parent)?;

        if self.parent != String::from("/") {
            self.content.push(FileItem {
                file_type: MetadataType::ReturnType,
                path: {
                    let mut path = Path::new(self.parent.as_str()).components();

                    path.next_back();

                    path.as_path().into()
                },
                name: String::new(),
                uuid: String::new(),
            });
        }

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => {
                    let file_type = MetadataType::from(res_path.file_type().unwrap());

                    if file_type == MetadataType::ErrorType {
                        continue;
                    };

                    self.content.push(FileItem {
                        name: res_path.file_name().to_str().unwrap().to_string(),
                        file_type: MetadataType::from(res_path.file_type().unwrap()),
                        path: res_path.path().into(),
                        uuid: String::new(),
                    })
                }
            }
        }

        Ok(())
    }

    // Generate a widget for rendering
    fn spawn_widget(&self) -> List {
        List::new(self.content.to_item_list(self.focused))
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
            )
    }
}

impl FileUI {
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

        self.local.focused = self.focus == FileUIFocus::Local;
        self.remote.focused = self.focus == FileUIFocus::Remote;

        // TODO : Change to only repopulate upon changes
        self.local.resolve_from_dir()?;
        self.remote.resolve_from_db(db)?;

        self.local.content.render_height = layout[0].height.into();
        self.remote.content.render_height = layout[1].height.into();

        self.local.content.render_height -= WIDGET_OFFSET;
        self.remote.content.render_height -= WIDGET_OFFSET;

        f.render_widget(self.local.spawn_widget(), layout[0]);
        f.render_widget(self.remote.spawn_widget(), layout[1]);

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
            FileUIFocus::Local => self.local.content.cusor_move(direction),
            FileUIFocus::Remote => self.remote.content.cusor_move(direction),
        };
    }

    pub fn expand_selection(&mut self) -> Result<(), intern_error::Error> {
        let focused_block = match self.focus {
            FileUIFocus::Local => &mut self.local,
            FileUIFocus::Remote => &mut self.remote,
        };

        let mut updated = false;

        focused_block.parent = match focused_block.content.get(focused_block.content.cursor_idx) {
            Some(val) => match val.file_type {
                MetadataType::CollectionType | MetadataType::ReturnType => {
                    updated = true;
                    match self.focus {
                        FileUIFocus::Local => (*val.path).display().to_string(),
                        FileUIFocus::Remote => (*val.uuid).to_string(),
                    }
                }
                _ => focused_block.parent.clone(),
            },
            None => {
                return Err(intern_error::Error::OutOfBoundsError(
                    focused_block.content.cursor_idx,
                ))
            }
        };

        if updated {
            focused_block.content.cursor_idx = 0;
        }

        Ok(())
    }
}

// Helper function to give a FileUI struct
pub fn file_ui(
    local_parent: &str,
    remote_parent: &str,
    db: &Connection,
) -> Result<FileUI, crate::intern_error::Error> {
    let mut ui = FileUI {
        local: FSBlock {
            name: String::from("local"),
            parent: local_parent.to_string(),
            content: FileList::new(),
            focused: true,
            ..Default::default()
        },
        remote: FSBlock {
            name: String::from("remote"),
            parent: remote_parent.to_string(),
            content: FileList::new(),
            focused: false,
            ..Default::default()
        },
        focus: FileUIFocus::Local,
    };

    ui.remote.resolve_from_db(db)?;
    ui.local.resolve_from_dir()?;

    Ok(ui)
}
