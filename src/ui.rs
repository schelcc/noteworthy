use std::{cmp::Ordering, fs};

use rusqlite::{named_params, Connection};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub enum Display {
    LocalRemoteSplit,
}

pub struct CursorPos {
    col: usize,
    row: usize,
}

pub struct FileItem {
    uuid: String,
    name: String,
    highlighted: bool,
}

pub struct FileList {
    content: Vec<FileItem>,
    cursor_idx: usize,
}

struct FSBlock {
    name: String,
    parent: String,
    content: FileList,
    // cursor: CursorPos,
}

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
    fn push(&mut self, value: FileItem) {
        self.content.push(value);
    }

    fn new() -> Self {
        FileList {
            content: Vec::new(),
            cursor_idx: 0,
        }
    }

    pub fn cusor_move(&mut self, direction: CursorDirection) {
        let delta = match direction {
            CursorDirection::Up => -1,
            CursorDirection::Down => 1,
        };

        self.cursor_idx = match self.cursor_idx.checked_add_signed(delta) {
            Some(val) => val,
            None => self.cursor_idx,
        };
    }
}

impl From<&FileList> for Vec<ListItem<'_>> {
    fn from(value: &FileList) -> Self {
        let mut result: Vec<ListItem> = Vec::new();

        for (idx, item) in value.content.iter().enumerate() {
            match idx.cmp(&value.cursor_idx) {
                Ordering::Equal => result.push(
                    ListItem::new(item.name.clone())
                        .style(Style::default().add_modifier(Modifier::UNDERLINED)),
                ),
                _ => result.push(ListItem::new(item.name.clone())),
            }
        }

        result
    }
}

impl FSBlock {
    fn resolve_from_db(&mut self, db: &Connection) -> Result<(), crate::intern_error::Error> {
        let mut stmt = db.prepare("SELECT uuid, name FROM objects WHERE parent=?1")?;

        let file_iter = stmt.query_map([&self.parent], |r| {
            // TODO : Update error handling
            Ok(FileItem {
                uuid: match r.get(0) {
                    Err(_) => String::from("error"),
                    Ok(res) => res,
                },
                name: match r.get(1) {
                    Err(_) => String::from("error"),
                    Ok(res) => res,
                },
                highlighted: false,
            })
        })?;

        for row in file_iter {
            if let Ok(row) = row {
                self.content.push(row);
            }
        }

        Ok(())
    }

    fn resolve_from_dir(&mut self) -> Result<(), crate::intern_error::Error> {
        let paths = fs::read_dir(&self.parent).expect("Fatal error: Couldn't read directory");

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => self.content.push(FileItem {
                    uuid: String::from(""),
                    name: res_path.path().display().to_string(),
                    highlighted: false,
                }),
            }
        }

        Ok(())
    }

    fn spawn_widget(&self) -> List {
        // TODO
        List::new(&self.content)
            .block(
                Block::default()
                    .title(self.name.clone())
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White))
    }
}

impl FileUI {
    pub fn render<B: Backend>(&self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

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
        },
        remote: FSBlock {
            name: String::from("remote"),
            parent: remote_parent.to_string(),
            content: FileList::new(),
        },
        focus: FileUIFocus::Local,
    };

    ui.remote.resolve_from_db(db)?;
    ui.local.resolve_from_dir()?;

    Ok(ui)
}
