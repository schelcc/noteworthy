use std::fs;

use rusqlite::{named_params, Connection};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct CursorPos {
    col: usize,
    row: usize,
}

pub struct FileItem {
    uuid: String,
    // name: String,
    highlighted: bool,
}

pub struct FileList {
    content: Vec<FileItem>
}

struct FSBlock {
    name: String,
    parent: String,
    content: FileList,
    // cursor: CursorPos,
}

pub struct FileUI {
    local: FSBlock,
    remote: FSBlock,
}

impl FileList {
    fn push(&mut self, value : FileItem) {
        self.content.push(value);
    }

    fn new() -> Self {
        FileList {
            content : Vec::new()
        }
    }
}

impl From<FileList> for Vec<ListItem<'_>> {
    fn from(value : FileList) -> Self {
        let mut result : Vec<ListItem> = Vec::new();

        for item in value.content {
            result.push(ListItem::new(item.uuid));
        };

        result
    }
}

impl FSBlock {
    fn resolve_from_db(&mut self, db: &Connection) -> Result<(), crate::intern_error::Error> {
        let mut stmt = db.prepare("SELECT uuid FROM objects WHERE parent=?1")?;

        let file_iter = stmt
            .query_map([&self.parent], |r| {
                Ok(FileItem {
                    uuid: match r.get(0) {
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
        };

        Ok(())
    }

    fn resolve_from_dir(&mut self) -> Result<(), crate::intern_error::Error> {
        let paths = fs::read_dir(&self.parent).expect("Fatal error: Couldn't read directory");

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => self.content.push(FileItem {
                    uuid: res_path.path().display().to_string(),
                    highlighted: false,
                }),
            }
        };



        Ok(())
    }

    fn spawn_widget(self) -> List<'static> {
        // TODO
        List::new(self.content)
            .block(Block::default().title(self.name).borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
    }
}

impl FileUI {
    pub fn render<B: Backend>(self, f: &mut Frame<B>) {
        let layout = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());

        f.render_widget(self.local.spawn_widget(), layout[0]);
        f.render_widget(self.remote.spawn_widget(), layout[1]);
    }
}

pub fn file_ui(local_parent: &str, remote_parent: &str, db: &Connection) -> Result<FileUI, crate::intern_error::Error> {
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
    };

    ui.remote.resolve_from_db(db)?;
    ui.local.resolve_from_dir()?;

    Ok(ui)
}
