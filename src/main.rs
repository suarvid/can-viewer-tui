mod draw_frame_table;
mod frame;
mod frame_filter;
mod ui;

use anyhow::Result;

use clap::Parser;
use crossterm::event::{self, KeyCode};
use crossterm::event::{Event, KeyEventKind};

use frame_filter::FrameIdFilter;
use ratatui::{prelude::*, widgets::*};

use std::io;
use std::time::{Duration, Instant};

use crate::frame::FrameCaptor;
use crate::ui::ui;

const APP_TITLE: &str = "CAN VIEWER TUI";
const DEFAULT_MAX_FRAMES_PER_SECOND: u32 = 1000;
const APP_TICK_RATE_MILLISECONDS: u64 = 100;
// Constant-size table to avoid performance degrading as more frames are captured
const APP_FRAMES_DISPLAYED_MAX_DEFAULT: usize = 500;

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
struct Args {
    /// Which can interface to listen to
    #[arg(short, long)]
    can_interface: String,
    /// CAN frame ID's to include in the resulting frame list, as hexadecimal values.
    /// If no ID's are given, all frames are included
    #[arg(short, long, default_value = None, value_parser, num_args = 1.., value_delimiter = ' ')]
    filter_frame_ids: Option<Vec<String>>,
    /// Max-value of the frames per second graph
    #[arg(short, long, default_value_t = DEFAULT_MAX_FRAMES_PER_SECOND)]
    max_frames_per_second_graph: u32,
    /// Maximum number of frames shown in the table at the same time
    #[arg(long, default_value_t = APP_FRAMES_DISPLAYED_MAX_DEFAULT)]
    frame_table_size: usize,
}

pub enum FrameView {
    FrameList,
    FrameSet,
}

/// Function pointer to function for drawing the main table of captured frames
type DrawFrameTableCallback = fn(&mut ratatui::Frame, app: &mut App, area: Rect);

pub struct App<'a> {
    pub frame_view: FrameView,
    pub table_state: TableState,
    pub title: &'a str,
    pub frames_per_second_max: u32,
    pub frame_id_filter: Option<FrameIdFilter>,
    pub frame_captor: FrameCaptor,
    pub enhanced_graphics: bool,
    pub row_color_main: Color,
    pub row_color_alt: Color,
    pub frames_displayed_max: usize,
    pub draw_frame_table: DrawFrameTableCallback,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        frames_per_second_max: u32,
        frames_displayed_max: usize,
        enhanced_graphics: bool,
        frame_captor: FrameCaptor,
    ) -> Self {
        App {
            frame_view: FrameView::FrameList,
            table_state: TableState::default().with_selected(0),
            title,
            frames_per_second_max,
            frame_id_filter: None,
            frame_captor,
            enhanced_graphics,
            row_color_main: Color::White,
            row_color_alt: Color::Gray,
            frames_displayed_max,
            draw_frame_table: draw_frame_table::draw_timestamped_frame_table,
        }
    }

    fn get_frame_table_len(&mut self) -> usize {
        match self.frame_view {
            FrameView::FrameList => self.frame_captor.get_captured_frames_list_len(),
            FrameView::FrameSet => self.frame_captor.get_captured_frames_set_len(),
        }
    }

    pub fn select_next_msg(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                let len = self.get_frame_table_len();
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
        let len = self.get_frame_table_len();
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

    pub fn select_latest_msg(&mut self) {
        self.table_state.select(Some(0));
    }

    pub fn toggle_frame_table_ui(&mut self) {
        match self.frame_view {
            FrameView::FrameList => {
                self.frame_view = FrameView::FrameSet;
                self.draw_frame_table = draw_frame_table::draw_counted_frame_set;
            }
            FrameView::FrameSet => {
                self.frame_view = FrameView::FrameList;
                self.draw_frame_table = draw_frame_table::draw_timestamped_frame_table;
            }
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
                            app.frame_captor.clear_captured_frames();
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.select_next_msg(),
                        KeyCode::Char('k') | KeyCode::Up => app.select_prev_msg(),
                        KeyCode::Char('t') => app.select_latest_msg(),
                        KeyCode::Char('v') => app.toggle_frame_table_ui(),
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn parse_filter_ids(raw_ids: Vec<String>) -> Vec<embedded_can::Id> {
    let mut filter_ids: Vec<embedded_can::Id> = Vec::new();

    for raw_id in raw_ids {
        match u32::from_str_radix(
            raw_id
                .strip_prefix("0x")
                .expect("Given filter CAN ID must be a hexadecimal string!"),
            16,
        ) {
            Ok(numeric_filter) => {
                if numeric_filter <= embedded_can::StandardId::MAX.as_raw().into() {
                    filter_ids.push(embedded_can::Id::Standard(
                        embedded_can::StandardId::new(numeric_filter as u16)
                            .expect("Failed to create Standard CAN ID for filtering!"),
                    ));
                } else if numeric_filter <= embedded_can::ExtendedId::MAX.as_raw() {
                    filter_ids.push(embedded_can::Id::Extended(
                        embedded_can::ExtendedId::new(numeric_filter)
                            .expect("Failed to create Extended CAN ID for filtering"),
                    ))
                }
            }
            Err(_) => panic!("Failed to parse filter CAN id: {}", raw_id),
        }
    }

    filter_ids
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut terminal = ratatui::init();
    terminal.clear()?;

    let frame_captor = FrameCaptor::new(args.can_interface.clone())?;

    let mut app = App::new(
        APP_TITLE,
        args.max_frames_per_second_graph,
        args.frame_table_size,
        false,
        frame_captor,
    );

    if let Some(filter_ids) = args.filter_frame_ids {
        app.frame_id_filter = Some(FrameIdFilter {
            ids: parse_filter_ids(filter_ids),
            filter_callback: frame_filter::filter_frame_on_ids,
        })
    }

    match run_app(
        &mut terminal,
        app,
        Duration::from_millis(APP_TICK_RATE_MILLISECONDS),
    ) {
        Ok(_) => {}
        Err(e) => eprintln!("Error occured when running application: {}", e),
    }

    ratatui::restore();

    Ok(())
}
