pub mod fs_interface;
pub mod ui;

use configparser::ini::Ini;
use rusqlite::Connection;
use std::{
    cmp::Ordering, collections::HashMap, error::Error, hash::Hash, io, thread, time::Duration,
};

use tui::{
    self,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use crossterm::{
    self,
    cursor::position,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use futures::{self, future::FusedFuture, select, FutureExt, StreamExt};
use futures_timer::Delay;

use async_std::{future, prelude::*};

use crate::ui::{FileUI, file_ui};

enum DispatchReturn {
    Exit,
}

fn main() -> Result<(), io::Error> {
    async_std::task::block_on(render_base());
    //    if let Err(why) = fs_interface::resolve_file_tree() {
    //        println!("[ERR] {:?}", why)
    //    };
    //
    Ok(())
}

async fn render_base() -> Result<(), io::Error> {
    // Setup + Initialization
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Alternate screen shit

    let backend = CrosstermBackend::new(stdout);
    let mut reader = EventStream::new();
    let mut terminal = Terminal::new(backend)?;

    
    let db = Connection::open_in_memory().expect("Fatal error: Failed to open SQLite database in memory");
    

    loop {
        // let mut delay = Delay::new(Duration::from_millis(1000)).fuse();
        let mut key_event = reader.next().fuse();
        let mut render_event = Delay::new(Duration::from_secs_f32(0.05)).fuse();

        select! {
            res_event = key_event => match res_event {
                Some(Ok(event)) => match event {
                    // KEY EVENT HANDLING
                    Event::Key(event) =>  {
                        match event.code {
                            KeyCode::Esc | KeyCode::Char('q') => break,
                            _ => ()
                        }
                    },
                    _ => (),
                },
                Some(Err(_)) | None => break,
            },
            _ = render_event => {
                terminal.draw(|f| {
                    file_ui("", "", &db).render(f);
                });
            }
        };
    }

    // Quit stuff

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
    )?;

    // terminal.show_cursor()?;

    Ok(())
}

// fn main_screen<B: Backend>(f: &mut Frame<B>, dbg_buf: &Vec<Spans>) {
//     // Get terminal dimensions to assist in multi-direciton layout
//     let dim = f.size();

//     // horizontal_layout = [navbar, main frame, footer]
//     let horizontal_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Percentage(5),
//                 Constraint::Percentage(90),
//                 Constraint::Percentage(5),
//             ]
//             .as_ref(),
//         )
//         .split(dim);

//     // split_layout = [local_fs, remote_fs]
//     let split_layout = Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
//         .split(horizontal_layout[1]);

//     // PLACEHOLDERS
//     let local_fs = Block::default()
//         .title("Local file system")
//         .borders(Borders::ALL);

//     let remote_fs = Block::default()
//         .title("reMarkable file system")
//         .borders(Borders::ALL);

//     f.render_widget(local_fs, split_layout[0]);
//     f.render_widget(remote_fs, split_layout[1]);
// }
