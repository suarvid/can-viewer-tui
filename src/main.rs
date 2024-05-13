mod frame;
mod ui;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::event::{Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::*;

use ratatui::backend::CrosstermBackend;
use ratatui::{prelude::*, widgets::*};

use socketcan::{CanSocket, Socket};

use std::error::Error;
use std::io;
use std::process::exit;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};

use crate::frame::{CapturedFrameInfo, FrameCaptor};
use crate::ui::ui;

pub struct App<'a> {
    pub table_state: TableState,
    pub title: &'a str,
    pub frame_info: Arc<Mutex<CapturedFrameInfo>>,
    pub enhanced_graphics: bool,
    pub row_color_main: Color,
    pub row_color_alt: Color,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        enhanced_graphics: bool,
        frame_info: Arc<Mutex<CapturedFrameInfo>>,
    ) -> Self {
        App {
            table_state: TableState::default().with_selected(0),
            title,
            frame_info,
            enhanced_graphics,
            row_color_main: Color::White,
            row_color_alt: Color::Gray,
        }
    }
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('c') => {
                            let mut frame_info = app.frame_info.lock().unwrap();
                            frame_info.clear_captured_frames();
                        },
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    } // loop
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <can-interface>", args[0]);
        eprintln!("Example: {} can0", args[0]);
        exit(1);
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let frame_info = Arc::new(Mutex::new(CapturedFrameInfo::new()));

    let rx_sock = CanSocket::open(args[1].as_str())?;
    let frame_captor = FrameCaptor::new(Arc::clone(&frame_info), rx_sock);
    thread::spawn(move || frame_captor.capture());

    let app = App::new("CAN Capture", false, Arc::clone(&frame_info));

    let res = run_app(&mut terminal, app, Duration::from_millis(100));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
