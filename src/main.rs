pub mod fs_interface;

use std::{io, thread, time::Duration, error::Error, collections::HashMap, cmp::Ordering};
use configparser::ini::Ini;

use tui::{self, 
    backend::{Backend, CrosstermBackend}, Frame, 
    layout::{Direction, Constraint, Layout, Rect}, 
    widgets::{Block, Borders, Paragraph, Wrap}, Terminal, style::{Modifier, Style, Color}, text::{Spans, Span}};

use crossterm::{self, 
    terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute, 
    event::{EnableMouseCapture, DisableMouseCapture, Event, EventStream, KeyCode, KeyEvent}, cursor::position};

use futures::{self, FutureExt, select, StreamExt, future::FusedFuture};
use futures_timer::Delay;

use async_std::{prelude::*, future};

enum DispatchReturn {
    Exit
}

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
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?; // Alternate screen shit

    let backend = CrosstermBackend::new(stdout);
    let mut reader = EventStream::new();
    let mut terminal = Terminal::new(backend)?;

    // Key event dispatch table
    // let mut key_dispatch : HashMap<KeyEvent, fn(KeyEvent) -> Option<DispatchReturn>> = HashMap::new();
    
    let mut dbg_buf:Vec<Spans> = Vec::new();

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
                    Event::Key(event) => {
                        if event == KeyCode::Esc.into() {
                            break
                        }

                        let event_str = format!("\r[INFO] {:?}", event);
                        dbg_buf.push(Spans::from(Span::raw(event_str)));
                    }, // Think about key map, for handling held down combinations?
                    Event::Mouse(_) => (),
                    _ => ()
                },
                Some(Err(_)) => break,
                None => break
            },
            _ = render_event => { 
                terminal.draw(|f| {main_screen(f, &dbg_buf)}); 
            }
        };

    }

    // Quit stuff

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    terminal.show_cursor()?;

    Ok(())
}

fn main_screen<B: Backend>(f: &mut Frame<'_, B>, dbg_buf:&Vec<Spans>) {
    // Get terminal dimensions to assist in multi-direciton layout
    let dim = f.size();

    let parent_layout = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage((80))]
        .as_ref()).split(dim);

    let nested_layout = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)]
        .as_ref()).split(parent_layout[1]);

    let file_tree_block = Block::default().title("File Tree").borders(Borders::ALL);
    f.render_widget(file_tree_block, parent_layout[0]);

    let placeholder = Block::default().title("Placeholder").borders(Borders::ALL);
    f.render_widget(placeholder, nested_layout[0]);

    let console_height:usize = nested_layout[1].height.into();
    let buf_length = &dbg_buf.len();

    let buf_clone = match console_height.cmp(&buf_length) {
        Ordering::Greater | Ordering::Equal => dbg_buf.clone(),
        Ordering::Less => dbg_buf.clone()[buf_length-console_height+2..].to_vec()
    };

    let debug_console = Paragraph::new(buf_clone)
            .block(Block::default().title("Debug Console").borders(Borders::ALL))
            .wrap(Wrap {trim: true});

    f.render_widget(debug_console,nested_layout[1]);
}

