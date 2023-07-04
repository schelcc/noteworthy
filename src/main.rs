/*
This file is part of Noteworthy.

Noteworthy is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License
as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Noteworthy is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with Noteworthy. If not, see <https://www.gnu.org/licenses/>.
*/

pub mod config;
pub mod fs_interface;
pub mod intern_error;
pub mod notification;
pub mod remote;
pub mod ui;

use fs_interface::resolve_file_tree;
use intern_error::Error;
use rusqlite::Connection;
use std::{
    io::{self, Stdout},
    sync::Arc,
};

use tui::{self, backend::CrosstermBackend, Terminal};

use crossterm::{
    self, cursor,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, PopKeyboardEnhancementFlags},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    notification::{NotificationType, NotificationWidget},
    ui::file_ui,
};

fn main() -> Result<(), crate::intern_error::Error> {
    // Setup + Initialization
    let mut stdout = io::stdout();

    enable_raw_mode()?;

    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Show
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let db = Connection::open_in_memory()?;

    // Main render loop
    let conclusion = async_std::task::block_on(render_base(&mut terminal, db));

    // De-init + Exit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
    )?;

    terminal.show_cursor()?;

    println!("{:?}", conclusion);

    Ok(())
}

async fn render_base(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    db: Connection,
) -> Result<(), crate::intern_error::Error> {
    // Key event reader
    // let mut reader = EventStream::new();

    // Atomic reference counter for db, allowing db to be passed around without
    // adding complexity via lifetimes
    let arc_db = Arc::new(db);

    // Resolve the current file tree into a db in memory
    resolve_file_tree(Arc::clone(&arc_db))?;

    let conclusion: Result<(), Error> = Ok(());

    let mut selected_ui = file_ui(Arc::clone(&arc_db))?;

    let mut notification_queue: Vec<NotificationWidget> = Vec::new();

    loop {
        let event = crossterm::event::read()?;
        // let mut render_event = Delay::new(Duration::from_secs_f32(0.05)).fuse();

        match event {
            Event::Key(event) => {
                match event.code {
                    // Global key responses
                    // TODO: Add flag to disable keyevent handling for text input
                    KeyCode::Esc | KeyCode::Char('q') => break Ok(()),
                    KeyCode::Char(' ') => {
                        notification_queue.pop();
                    }
                    _ => {
                        // Don't handle context-specific keys if a notification has yet to be dismissed
                        if notification_queue.len() == 0 {
                            soft_error_recovery(
                                &mut notification_queue,
                                selected_ui.key_handler(event.code),
                            )?;
                        }
                    }
                }
            }
            _ => (),
        };

        let mut render_result: Result<(), Error> = Ok(());

        terminal.draw(|f| {
            render_result = selected_ui.render(f);

            match notification_queue.last() {
                Some(notif) => {
                    notif.render(f);
                }
                None => (),
            };
        })?;

        match soft_error_recovery(&mut notification_queue, render_result) {
            Err(why) => break Err(why),
            Ok(_) => (),
        };
    }?;

    conclusion
}

// TODO: Move to intern_error.rs and somehow have a selection of errors we want to recover
// from and ones we dont
fn soft_error_recovery<T>(
    notifications: &mut Vec<NotificationWidget>,
    result: Result<T, Error>,
) -> Result<Option<T>, Error> {
    match result {
        Err(why) => {
            notifications.push(
                NotificationWidget::default()
                    .text(why.to_string().as_str())
                    .notif_type(NotificationType::ErrorLow),
            );
            Ok(None)
        }
        Ok(val) => Ok(Some(val)),
    }
}
