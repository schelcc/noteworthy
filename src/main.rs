pub mod config;
pub mod fs_interface;
pub mod intern_error;
pub mod notification;
pub mod remote;
pub mod ui;

use fs_interface::resolve_file_tree;
use rusqlite::Connection;
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};

use tui::{self, backend::CrosstermBackend, Terminal};

use crossterm::{
    self, cursor,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode,
        PopKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use futures::{self, select, FutureExt, StreamExt};
use futures_timer::Delay;

use crate::ui::file_ui;

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

    terminal.show_cursor()?;

    let db = Connection::open_in_memory()?;

    let conclusion = async_std::task::block_on(render_base(&mut terminal, db));

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
    let mut reader = EventStream::new();

    let arc_db = Arc::new(db);

    resolve_file_tree(Arc::clone(&arc_db))?;

    let conclusion: Result<(), crate::intern_error::Error> = Ok(());

    // Should be based on config
    let mut ui = file_ui(Arc::clone(&arc_db))?;

    loop {
        let mut key_event = reader.next().fuse();
        let mut render_event = Delay::new(Duration::from_secs_f32(0.05)).fuse();

        select! {
            res_event = key_event => match res_event {
                Some(Ok(event)) => match event {
                    // KEY EVENT HANDLING
                    Event::Key(event) =>  {
                        match event.code {
                            KeyCode::Esc | KeyCode::Char('q') => break Ok(()),
                            KeyCode::Up => {ui.cursor_move(ui::CursorDirection::Up);},
                            KeyCode::Down => {ui.cursor_move(ui::CursorDirection::Down);},
                            KeyCode::Tab => {ui.toggle_focus();},
                            KeyCode::Enter => {ui.expand_selection()?;},
                            KeyCode::PageDown => {ui.cursor_move(ui::CursorDirection::PgDn);},
                            KeyCode::PageUp => {ui.cursor_move(ui::CursorDirection::PgUp);},
                            _ => ()
                        }
                    },
                    _ => (),
                },
                Some(Err(why)) => break Err(why.into()),
                None => break Err(crate::intern_error::Error::CrosstermError(String::from("None KeyEvent"))),
            },
            _ = render_event => {
                let mut render_result : Result<(), intern_error::Error> = Ok(());

                terminal.draw(|f| {
                    render_result = ui.render(f);
                })?;

                render_result?;
            }
        };
    }?;

    conclusion
}
