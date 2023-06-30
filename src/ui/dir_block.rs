use std::{cmp::Ordering, fs, path::Path, sync::Arc};

use tui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::{
    fs_interface::MetadataType,
    intern_error::{self, Error},
};

use super::{super::config, block::FSListBlock, file_item::FileItem, CursorDirection};

pub struct DirBlock {
    name: String,
    parent: Box<Path>,
    cursor_idx: usize,
    pub focused: bool,
    content: Vec<FileItem>,
}

impl FSListBlock for DirBlock {
    fn new(title: &'static str, _: Option<Arc<rusqlite::Connection>>) -> Self {
        DirBlock {
            name: String::from(title),
            parent: Path::new("/home/").into(),
            focused: false,
            cursor_idx: 0,
            content: Vec::new(),
        }
    }

    fn resolve(&mut self) -> Result<(), intern_error::Error> {
        self.content = Vec::new();

        let paths = fs::read_dir(&self.parent)?;

        if self.parent != Path::new("/").into() {
            self.content
                .push(FileItem::new().file_type(MetadataType::ReturnType).path({
                    let path = &mut self.parent.components();

                    path.next_back();

                    path.as_path().into()
                }));
        };

        for path in paths {
            match path {
                Err(_) => (),
                Ok(res_path) => {
                    let file_type = MetadataType::from(res_path.file_type().unwrap());

                    if file_type == MetadataType::ErrorType {
                        continue;
                    };

                    self.content.push(
                        FileItem::new()
                            .name(res_path.file_name().to_str().unwrap())
                            .file_type(MetadataType::from(res_path.file_type().unwrap()))
                            .path(res_path.path().into()), // Fix
                    );
                }
            }
        }

        Ok(())
    }

    fn generate_list(
        &self,
        render_area: Rect,
    ) -> Result<Vec<ListItem<'static>>, intern_error::Error> {
        let mut result = Vec::new();

        let adj_height = render_area.height - super::WIDGET_OFFSET;

        let offset: Option<usize> = match self.cursor_idx.cmp(&adj_height.into()) {
            Ordering::Greater => Some(self.cursor_idx - usize::from(adj_height)),
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

            let mut file_name = match item.file_type {
                MetadataType::CollectionType => String::from("/"),
                MetadataType::ReturnType => String::from("../"),
                _ => String::from(" "),
            };

            let mut style = match item.file_type {
                MetadataType::CollectionType | MetadataType::ReturnType => {
                    Style::default().add_modifier(Modifier::BOLD)
                }
                MetadataType::DocumentType => Style::default().add_modifier(Modifier::ITALIC),
                MetadataType::DefaultType | MetadataType::ErrorType => Style::default(),
            };

            if item.file_type == MetadataType::CollectionType
                || item.file_type == MetadataType::DocumentType
            {
                file_name.push_str(&item.name);
            };

            if self.focused && idx.cmp(&self.cursor_idx) == Ordering::Equal {
                style = style.add_modifier(Modifier::REVERSED)
            };

            result.push(ListItem::new(file_name).style(style));
        }

        Ok(result)
    }

    fn render(&mut self, render_area: Rect) -> Result<List, Error> {
        if self.focused {
            self.resolve()?;
        };

        Ok(List::new(self.generate_list(render_area).unwrap())
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
            ))
    }

    fn cursor_move(&mut self, direction: super::CursorDirection) {
        let delta: isize = match direction {
            CursorDirection::Down => 1,
            CursorDirection::Up => -1,
            CursorDirection::PgDn => 15,
            CursorDirection::PgUp => -15,
        };

        self.cursor_idx = match self.cursor_idx.checked_add_signed(delta) {
            Some(val) => match val.cmp(&self.content.len()) {
                Ordering::Less => val,
                _ => self.cursor_idx,
            },
            None => self.cursor_idx,
        };
    }

    fn expand_selection(&mut self) -> () {
        let selected_file = match self.content.get(self.cursor_idx) {
            Some(val) => val,
            None => return (),
        };

        match selected_file.file_type {
            MetadataType::CollectionType | MetadataType::ReturnType => {
                self.parent = selected_file.path.clone().into();
                self.cursor_idx = 0;
            }
            _ => (),
        };
    }
}
