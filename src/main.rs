pub mod fs_interface;

use std::{io, thread, time::Duration, error::Error};
use configparser::ini::Ini;

use tui::{self, 
    backend::{Backend, CrosstermBackend}, Frame, 
    layout::{Direction, Constraint, Layout}, 
    widgets::{Block, Borders}, Terminal};

use crossterm::{self, 
    terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute, 
    event::{EnableMouseCapture, DisableMouseCapture, Event, EventStream, KeyCode}, cursor::position};

use futures::{self, FutureExt, select, StreamExt, future::FusedFuture};
use futures_timer::Delay;

use async_std::{prelude::*, future};

fn main() -> Result<(), io::Error> {
    // enable_raw_mode()?;

    // // Prep stuff
    // let mut stdout = io::stdout();

    // execute!(stdout, EnableMouseCapture)?;

    // let backend = CrosstermBackend::new(stdout);

    // let mut terminal = Terminal::new(backend)?;

    // // Render
    // // terminal.draw(|f| main_screen(f))?;

    // // Allow render to stick for 5 sec
    // // thread::sleep(Duration::from_millis(5000));
    // // async_std::task::block_on(event_handler());

    // // Exit stuff
    // disable_raw_mode()?;

    // execute!(terminal.backend_mut(), DisableMouseCapture)?;

    // terminal.show_cursor()?;

    async_std::task::block_on(render_base());

    Ok(())
}

async fn render_base() -> Result<(), io::Error> {

    // Setup + Initialization
    let mut stdout = io::stdout();

    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture); // Alternate screen shit

    let backend = CrosstermBackend::new(stdout);
    let mut reader = EventStream::new();
    let mut terminal = Terminal::new(backend)?;

    loop {
        // let mut delay = Delay::new(Duration::from_millis(1000)).fuse();
        let mut key_event = reader.next().fuse();
        let mut render_event = Delay::new(Duration::from_secs_f32(0.05))
            .fuse();

        let mut last_key_event:Event;

        select! {
            res_event = key_event => match res_event {
                Some(Ok(event)) => match event {
                    // KEY EVENT HANDLING
                    Event::Key(_) => (), // Think about key map, for handling held down combinations?
                    Event::Mouse(_) => (),
                    _ => ()
                },
                Some(Err(_)) => break,
                None => break
            },
            _ = render_event => { 
                terminal.draw(|f| {main_screen(f);}); 
            }
        };
    }

    // Quit stuff

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    terminal.show_cursor()?;

    Ok(())
}

fn main_screen<B: Backend>(f: &mut Frame<'_, B>) {
    let chunks = Layout::default().direction(Direction::Horizontal).margin(1)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(80)
        ].as_ref()).split(f.size());

    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[0]);
    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, chunks[1]);
}

