pub mod fs_interface;
pub mod intern_error;
pub mod ui;

use fs_interface::resolve_file_tree;
use rusqlite::Connection;
use std::{io::{self, Stdout}, time::Duration};

use tui::{self, backend::CrosstermBackend, Terminal};

use crossterm::{
    self,
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

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Alternate screen shit

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let db = Connection::open_in_memory()?;


    let conclusion = async_std::task::block_on(render_base(&mut terminal, &db));

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

async fn render_base(terminal : &mut Terminal<CrosstermBackend<Stdout>>, db : &Connection) -> Result<(), crate::intern_error::Error> {
    let mut reader = EventStream::new();

    resolve_file_tree(&db)?;

    let conclusion : Result<(), crate::intern_error::Error>;

    match loop {
        let mut key_event = reader.next().fuse();
        let mut render_event = Delay::new(Duration::from_secs_f32(0.05)).fuse();

        select! {
            res_event = key_event => match res_event {
                Some(Ok(event)) => match event {
                    // KEY EVENT HANDLING
                    Event::Key(event) =>  {
                        match event.code {
                            KeyCode::Esc | KeyCode::Char('q') => break Ok(()),
                            _ => ()
                        }
                    },
                    _ => (),
                },
                Some(Err(why)) => break Err(why.into()),
                None => break Err(crate::intern_error::Error::CrosstermError(String::from("None KeyEvent"))),
            },
            _ = render_event => {
                terminal.draw(|f| {
                    match file_ui("/home/schelcc/", "root", &db) {
                        Err(_) => (),
                        Ok(ui) => ui.render(f)
                    }
                })?;
            }
        };
    } {
        Err(why) => {conclusion = Err(why)},
        Ok(_) => {conclusion = Ok(())}
    };

    // Quit stuff



    // terminal.show_cursor()?;

    conclusion
}
