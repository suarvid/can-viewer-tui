
use embedded_can::Frame;
use ratatui::layout::Constraint::{Fill, Percentage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::block::Title;
use ratatui::widgets::Cell;
use ratatui::{prelude::*, widgets::*};
use socketcan::{CanFrame}; //, Frame};

//use crate::frame::CapturedFrame;
use crate::App;

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Percentage(5), Fill(90)])
        .split(f.size());

    let keybindings = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
        " Clear Frame Info ".into(),
        "<C> ".blue().bold(),
    ]));

    draw_captured_frames(f, app, rects[1], keybindings);

    //let frame_info = app.frame_info.lock().unwrap();
    let frame_info = app.frame_captor.get_captured_frames();
    let n_unique_frames = app.frame_captor.get_captured_frames_len(); //frame_info.captured_frame_set.len();
    let n_total_frames = app.frame_captor.get_total_frame_count();
    let frames_per_second = app.frame_captor.get_frames_per_second();

    draw_header(
        f,
        rects[0],
        n_total_frames,
        n_unique_frames,
        frames_per_second,
    );
}

// TODO: Implement this to allow switching between Hex and Dec string representations
// fn frame_data_as_str(_frame: &CapturedFrame, _app: &App) {}

fn add_row_for_frame(
    i: usize,
    frame: &CanFrame,
    rows: &mut Vec<Row>,
    row_color_main: Color,
    row_color_alt: Color,
) {
    let color = match i % 2 {
        0 => row_color_main,
        _ => row_color_alt,
    };

    let mut cells = vec![];
    cells.push(Cell::from(Text::from(format!("{:x?}", CanFrame::id(frame)))));
    cells.push(Cell::from(Text::from(format!(
        "{:?}",
        CanFrame::dlc(frame)
    ))));
    cells.push(Cell::from(Text::from(format!(
        "{}",
        CanFrame::is_extended(frame)
    ))));
    cells.push(Cell::from(Text::from(format!(
        "{:?}",
        CanFrame::data(frame)
    ))));
    rows.push(
        Row::new(cells)
            .style(Style::default().fg(Color::Black).bg(color))
            .height(1),
    );
}

// TODO: A lil ugly that this is the place responsible for drawing keybindings
fn draw_captured_frames(f: &mut ratatui::Frame, app: &mut App, area: Rect, keybindings: Title) {
    let header_style = Style::default().fg(Color::White).bg(Color::Black);

    let selected_style = Style::default().fg(Color::Black).bg(Color::LightYellow);

    let header = ["ID", "DLC", "Extended", "Data"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let mut rows: Vec<Row> = Vec::new();

    match app.frame_captor.get_captured_frames() {
        crate::frame::CapturedFrames::List(vec) => {
            for (i, frame) in vec.iter().enumerate() {
                add_row_for_frame(i, frame, &mut rows, app.row_color_main, app.row_color_alt);
            }
        }
        crate::frame::CapturedFrames::Set(hash_map) => {
            for (i, (_, frame)) in hash_map.iter().enumerate() {
                add_row_for_frame(i, frame, &mut rows, app.row_color_main, app.row_color_alt);
            }
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .highlight_style(selected_style)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_set(border::THICK)
            .title(
                keybindings
                    .alignment(Alignment::Center)
                    .position(block::Position::Bottom),
            ),
    );
    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_header(
    f: &mut ratatui::Frame,
    area: Rect,
    total_frame_count: usize,
    n_unique_frames: usize,
    frames_per_second: usize,
) {
    let footer = Paragraph::new(Line::from(format!(
        "Unique Frame IDs: {}, Total Frame Count {}, Frames Per Second: {}",
        n_unique_frames, total_frame_count, frames_per_second
    )))
    .centered()
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double),
    );

    f.render_widget(footer, area);
}
