use std::fs;

use rusqlite::{named_params, Connection};
use tui::{
    backend::Backend,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, ListItem, List},
    Frame, style::{Style, Color, Modifier},
};

pub struct CursorPos {
    col: usize,
    row: usize,
}

pub struct ListFile {
    uuid: String,
    // name: String,
    highlighted: bool,
}

struct FSBlock {
    name: String,
    parent: String,
    content: Vec<ListFile>,
    // cursor: CursorPos,
}

pub struct FileUI {
    local: FSBlock,
    remote: FSBlock,
}

impl FSBlock {
    fn resolve_from_db(&mut self, db: &Connection) {
        let mut stmt = match db
            .prepare(format!("SELECT uuid FROM objects WHERE parent={}", self.parent).as_str())
        {
            Err(why) => panic!("{}", why),
            Ok(res) => res,
        };

        let file_iter = stmt
            .query_map([], |r| {
                Ok(ListFile {
                    uuid: match r.get(0) {
                        Err(_) => String::from("error"),
                        Ok(res) => res,
                    },
                    highlighted: false,
                })
            })
            .expect("Fatal error: stuff");

        for row in file_iter {
            if let Ok(row) = row {
                self.content.push(row);
            }
        }
    }

    fn resolve_from_dir(&mut self) {
        // TODO: Handle error
        let paths = fs::read_dir(&self.parent).expect("Fatal error: Couldn't read directory");

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => self.content.push(ListFile { uuid: res_path.path().display().to_string(), highlighted: false })
            }
        }
    }

    fn spawn_widget(&self) -> Block {
        // TODO
        List::new(self.content).block(Block::default().title(self.title).borders(Borders::ALL)).style(Style::default().fg(Color::White)).highlight_style(Style::default().add_modifier(Modifier::ITALIC)).highlight_symbol(">>");
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
}

pub fn file_ui(local_parent: &str, remote_parent: &str, db: &Connection) -> FileUI {
    let mut ui = FileUI {
        local: FSBlock {
            name: String::from("local"),
            parent: local_parent.to_string(),
            content: Vec::new(),
        },
        remote: FSBlock {
            name: String::from("remote"),
            parent: remote_parent.to_string(),
            content: Vec::new(),
        },
    };

    ui.remote.resolve_from_db(db);

    ui
}
