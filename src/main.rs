mod frame;
mod ui;

use anyhow::Result;

use crossterm::event::{self, KeyCode};
use crossterm::event::{Event, KeyEventKind};

use ratatui::{prelude::*, widgets::*};

use std::env;
use std::io;
use std::process::exit;
use std::time::{Duration, Instant};

use crate::frame::FrameCaptor;
use crate::ui::ui;

pub struct App<'a> {
    pub table_state: TableState,
    pub title: &'a str,
    pub frames_per_second_max: u32,
    pub frame_id_filter: Option<&'a str>,
    pub frame_captor: FrameCaptor,
    pub enhanced_graphics: bool,
    pub row_color_main: Color,
    pub row_color_alt: Color,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        frames_per_second_max: u32,
        enhanced_graphics: bool,
        frame_captor: FrameCaptor,
    ) -> Self {
        App {
            table_state: TableState::default().with_selected(0),
            title,
            frames_per_second_max,
            frame_id_filter: None,
            frame_captor,
            enhanced_graphics,
            row_color_main: Color::White,
            row_color_alt: Color::Gray,
        }
    }

    pub fn select_next_msg(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                let len = self.frame_captor.get_captured_frames_len();
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn select_prev_msg(&mut self) {
        let len = self.frame_captor.get_captured_frames_len();
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => len - 1,
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
                            app.frame_captor.clear_captured_frames();
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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <can-interface> <max-frames-per-second>", args[0]);
        eprintln!("Example: {} can0 1000", args[0]);
        exit(1);
    }

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let frame_captor = FrameCaptor::new(args[1].clone())?;

    let max_fps: u32 = args[2].parse()?;

    let app = App::new("CAN Capture", max_fps, false, frame_captor);

    match run_app(&mut terminal, app, Duration::from_millis(1000)) {
        Ok(_) => {}
        Err(e) => eprintln!("Error occured when running application: {}", e),
    }

    ratatui::restore();

    Ok(())
}
