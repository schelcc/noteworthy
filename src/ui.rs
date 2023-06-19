use std::{cmp::Ordering, fs};

use rusqlite::{
    named_params,
    types::{FromSql, FromSqlError},
    Connection, ToSql,
};

use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::fs_interface::MetadataType;

#[derive(Default)]
pub enum Display {
    #[default]
    LocalRemoteSplit,
}

#[derive(Default)]
pub struct FileItem {
    uuid: String,
    name: String,
    file_type: MetadataType,
}

#[derive(Default)]
pub struct FileList {
    content: Vec<FileItem>,
    cursor_idx: usize,
}

#[derive(Default)]
struct FSBlock {
    name: String,
    parent: String,
    content: FileList,
    focused: bool,
}

#[derive(PartialEq)]
enum FileUIFocus {
    Local,
    Remote,
}

pub enum CursorDirection {
    Up,
    Down,
}

pub struct FileUI {
    local: FSBlock,
    remote: FSBlock,
    focus: FileUIFocus,
}

impl FileList {
    // Forward .push() for wrapper struct
    fn push(&mut self, value: FileItem) {
        self.content.push(value);
    }

    fn new() -> Self {
        FileList {
            content: Vec::new(),
            cursor_idx: 0,
        }
    }

    // Adjust the currently selected row dependent on the CursorDirection struct
    pub fn cusor_move(&mut self, direction: CursorDirection) {
        let delta = match direction {
            CursorDirection::Up => -1,
            CursorDirection::Down => 1,
        };

        // Ensure cursor doesn't try to go to a row that doesn't exist
        self.cursor_idx = match self.cursor_idx.checked_add_signed(delta) {
            Some(val) => match &val.cmp(&self.content.len()) {
                Ordering::Less => val,
                Ordering::Equal | Ordering::Greater => self.cursor_idx,
            },
            None => self.cursor_idx,
        };
    }

    // Convert FileList to Vec<ListItem> for tui to generate an array of ListItems from
    // Applies cursor styling, depends on focus bool, hiding underline if not focused
    pub fn to_item_list(&self, focus: bool) -> Vec<ListItem> {
        let mut result: Vec<ListItem> = Vec::new();

        for (idx, item) in self.content.iter().enumerate() {
            // Add "/" prefix to any CollectionTypes to denote directory
            let mut file_name = match item.file_type {
                MetadataType::CollectionType => String::from("/"),
                _ => String::from(""),
            };

            file_name.push_str(&item.name);

            // Push ListItem w/ underline if focused, otherwise just regular ListItem
            if focus && idx.cmp(&self.cursor_idx) == Ordering::Equal {
                result.push(
                    ListItem::new(file_name)
                        .style(Style::default().add_modifier(Modifier::UNDERLINED)),
                )
            } else {
                result.push(ListItem::new(file_name))
            }
        }

        result
    }
}

impl FSBlock {
    // Populate FileList from a rusqlite DB
    fn resolve_from_db(&mut self, db: &Connection) -> Result<(), crate::intern_error::Error> {
        let mut stmt = db.prepare("SELECT uuid, name, object_type FROM objects WHERE parent=?1")?;

        let file_iter = stmt.query_map([&self.parent], |r| {
            Ok(FileItem {
                uuid: r.get(0)?,
                name: r.get(1)?,
                file_type: MetadataType::from(r.get(2)?),
            })
        })?;

        for row in file_iter {
            if let Ok(row) = row {
                self.content.push(row);
            }
        }

        Ok(())
    }

    // Populate FileList from a directory
    fn resolve_from_dir(&mut self) -> Result<(), crate::intern_error::Error> {
        let paths = fs::read_dir(&self.parent)?;

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => self.content.push(FileItem {
                    name: res_path.path().display().to_string(),
                    ..Default::default()
                }),
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
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
    }
}

impl FileUI {
    // Create the layout and then render generated widgets
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        self.local.focused = self.focus == FileUIFocus::Local;
        self.remote.focused = self.focus == FileUIFocus::Remote;

        f.render_widget(self.local.spawn_widget(), layout[0]);
        f.render_widget(self.remote.spawn_widget(), layout[1]);
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
        },
        remote: FSBlock {
            name: String::from("remote"),
            parent: remote_parent.to_string(),
            content: FileList::new(),
            focused: false,
        },
        focus: FileUIFocus::Local,
    };

    ui.remote.resolve_from_db(db)?;
    ui.local.resolve_from_dir()?;

    Ok(ui)
}
