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

use crate::{
    fs_interface::{self, MetadataType},
    intern_error,
};

#[derive(Default)]
pub enum Display {
    #[default]
    LocalRemoteSplit,
}

#[derive(Default)]
pub struct FileItem {
    uuid: String,
    name: String,
    path: String,
    file_type: MetadataType,
}

#[derive(Default)]
pub struct FileList {
    content: Vec<FileItem>,
    cursor_idx: usize,
}

// #[derive(Default)]
struct FSBlock {
    name: String,
    parent: String,
    content: FileList,
    focused: bool,
    last_parents: Vec<String>,
}

impl Default for FSBlock {
    fn default() -> Self {
        FSBlock {
            name: String::default(),
            parent: String::default(),
            content: FileList::default(),
            focused: false,
            last_parents: Vec::new(),
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

    // Forward .get() for wrapper struct
    fn get(&self, idx: usize) -> Option<&FileItem> {
        self.content.get(idx)
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
                MetadataType::ErrorType => {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }
                MetadataType::DefaultType => {
                    style = style.add_modifier(Modifier::DIM);
                }
            }

            if item.file_type == MetadataType::CollectionType
                || item.file_type == MetadataType::DocumentType
            {
                file_name.push_str(&item.name);
            }

            // Push ListItem w/ underline if focused, otherwise just regular ListItem
            if focus && idx.cmp(&self.cursor_idx) == Ordering::Equal {
                style = style.add_modifier(Modifier::UNDERLINED);
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
                path: r.get(0)?,
                name: r.get(1)?,
                file_type: MetadataType::from(r.get::<usize, String>(2)?),
                ..Default::default()
            })
        })?;

        match db.query_row(
            "SELECT parent FROM objects WHERE uuid=:uuid",
            named_params! {":uuid":self.parent},
            |r| r.get::<usize, String>(0),
        ) {
            Err(_) => (),
            Ok(val) => {
                self.content.push(FileItem {
                    path: val,
                    file_type: MetadataType::ReturnType,
                    ..Default::default()
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

        self.content.push(FileItem {
            file_type: MetadataType::ReturnType,
            path: fs_interface::walk_back_path(self.parent.clone())?,
            ..Default::default()
        });

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => self.content.push(FileItem {
                    name: res_path.file_name().to_str().unwrap().to_string(),
                    file_type: MetadataType::from(res_path.file_type().unwrap()),
                    path: res_path.path().display().to_string(),
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
        // match self.focus {
        //     FileUIFocus::Local => {
        //         self.local.parent = match self.local.content.get(self.local.content.cursor_idx) {
        //             Some(val) => {
        //                 if val.file_type == MetadataType::CollectionType || val.file_type == MetadataType::ReturnType {
        //                     (*val.path).to_string()
        //                 } else {
        //                     self.local.parent
        //                 }
        //             },
        //             None => {
        //                 return Err(intern_error::Error::OutOfBoundsError(
        //                     self.local.content.cursor_idx,
        //                 ))
        //             }
        //         };
        //         self.local.content.cursor_idx = 0;
        //     }
        //     FileUIFocus::Remote => {
        //         self.remote.parent = match self.remote.content.get(self.remote.content.cursor_idx) {
        //             Some(val) => (*val.path).to_string(),
        //             None => {
        //                 return Err(intern_error::Error::OutOfBoundsError(
        //                     self.remote.content.cursor_idx,
        //                 ))
        //             }
        //         };

        //         self.remote.content.cursor_idx = 0;
        //     }
        // }

        let focused_block = match self.focus {
            FileUIFocus::Local => &mut self.local,
            FileUIFocus::Remote => &mut self.remote,
        };

        let mut updated = false;

        focused_block.parent = match focused_block.content.get(focused_block.content.cursor_idx) {
            Some(val) => match val.file_type {
                MetadataType::CollectionType | MetadataType::ReturnType => {
                    updated = true;
                    (*val.path).to_string()
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
