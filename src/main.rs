mod frame;
mod ui;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode};
use crossterm::event::{Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::*;

use frame::CapturedFrames;
use ratatui::backend::CrosstermBackend;
use ratatui::{prelude::*, widgets::*};

use socketcan::{CanSocket, Socket};

use std::error::Error;
use std::io;
use std::process::exit;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};

use crate::frame::FrameCaptor;
use crate::ui::ui;

pub struct App<'a> {
    pub table_state: TableState,
    pub title: &'a str,
    //pub frame_info: Arc<Mutex<CapturedFrameInfo>>,
    pub frame_captor: Arc<Mutex<FrameCaptor>>,
    pub enhanced_graphics: bool,
    pub row_color_main: Color,
    pub row_color_alt: Color,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        enhanced_graphics: bool,
        frame_captor: Arc<Mutex<FrameCaptor>>,
        //frame_info: Arc<Mutex<CapturedFrameInfo>>,
    ) -> Self {
        App {
            table_state: TableState::default().with_selected(0),
            title,
            //frame_info,
            frame_captor,
            enhanced_graphics,
            row_color_main: Color::White,
            row_color_alt: Color::Gray,
        }
    }

    pub fn select_next_msg(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => match self.frame_captor.lock().unwrap().get_captured_frames() {
                CapturedFrames::List(l) => l.len(),
                CapturedFrames::Set(s) => s.len(),
            },
            None => 0,
            //Some(i) => {
            //if i >= self.frame_info.lock().unwrap().captured_frame_set.len() - 1 {
            //0
            //} else {
            //i + 1
            //}
            //}
            //None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn select_prev_msg(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.frame_info.lock().unwrap().captured_frame_set.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.frame_info.lock().unwrap().captured_frame_set.len() - 1,
        };

        self.table_state.select(Some(i));
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
                            app.frame_captor
                                .lock()
                                .expect("PANG!")
                                .clear_captured_frames();
                            //app.frame_captor.clear_captured_frames();
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.select_next_msg(),
                        KeyCode::Char('k') | KeyCode::Up => app.select_prev_msg(),
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

    match CanSocket::open(args[1].as_str()) {
        Ok(rx_sock) => {
            let frame_captor = FrameCaptor::new(rx_sock);
            //let frame_captor = FrameCaptor::new(Arc::clone(&frame_info), rx_sock);
            thread::spawn(|| frame_captor.capture());

            let app = App::new("CAN Capture", false, Arc::new(Mutex::new(frame_captor)));

            match run_app(&mut terminal, app, Duration::from_millis(100)) {
                Ok(_) => {}
                Err(e) => eprintln!("Error occured when running application: {}", e),
            }
        }
        Err(e) => eprintln!(
            "Failed to open can interface {}. Reason: {}",
            args[1].as_str(),
            e
        ),
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
