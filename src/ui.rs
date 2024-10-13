use embedded_can::Frame;
use ratatui::layout::Constraint::{Fill, Percentage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::block::Title;
use ratatui::widgets::Cell;
use ratatui::{prelude::*, widgets::*};
use socketcan::CanFrame;

use crate::frame::TimestampedFrame;
use crate::App;

pub fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let rects = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Percentage(5), Fill(90)])
        .split(f.area());

    let keybindings = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
        " Clear Frame Info ".into(),
        "<C> ".blue().bold(),
    ]));

    draw_captured_frames(f, app, rects[1], keybindings);

    let n_unique_frames = app.frame_captor.get_captured_frames_len();
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

fn get_row_for_timestamped_frame(frame: &TimestampedFrame) -> Vec<Cell<'_>> {
    let mut cells = vec![];
    cells.push(Cell::from(Text::from(format!("{}", frame.get_timestamp()))));
    cells.push(Cell::from(Text::from(format!(
        "0x{:x}",
        frame.get_numeric_id()
    ))));
    cells.push(Cell::from(Text::from(format!(
        "{:?}",
        CanFrame::dlc(&frame.frame)
    ))));
    cells.push(Cell::from(Text::from(format!(
        "{}",
        CanFrame::is_extended(&frame.frame)
    ))));
    cells.push(Cell::from(Text::from(format!(
        "{:?}",
        CanFrame::data(&frame.frame)
    ))));

    cells
}

fn get_header_for_timestamped_frames(header_style: Style) -> Row<'static> {
    ["Timestamp", "ID", "DLC", "Extended", "Data"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
}

fn draw_timestamped_frames(
    rows: Vec<Row<'_>>,
    header_style: Style,
    selected_style: Style,
    keybindings: Title<'_>,
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    app: &mut App<'_>,
) {
    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Percentage(5),
            Constraint::Percentage(10),
            Constraint::Percentage(5),
            Constraint::Fill(1),
        ],
    )
    .header(get_header_for_timestamped_frames(header_style))
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


// TODO: A lil ugly that this is the place responsible for drawing keybindings
fn draw_captured_frames(f: &mut ratatui::Frame, app: &mut App, area: Rect, keybindings: Title) {
    let header_style = Style::default().fg(Color::White).bg(Color::Black);
    let selected_style = Style::default().fg(Color::Black).bg(Color::LightYellow);
    let mut rows: Vec<Row> = Vec::new();

    match app.frame_captor.get_captured_frames() {
        crate::frame::CapturedFrames::List(vec) => {
            for (i, frame) in vec.iter().enumerate() {
                let color = match i % 2 {
                    0 => app.row_color_main,
                    _ => app.row_color_alt,
                };

                let cells = get_row_for_timestamped_frame(frame);
                rows.push(Row::new(cells).style(Style::default().fg(Color::Black).bg(color)));
            }
            draw_timestamped_frames(
                rows,
                header_style,
                selected_style,
                keybindings,
                f,
                area,
                app,
            );
        }
        crate::frame::CapturedFrames::Set(_hash_map) => {
            todo!("Add support for drawing set of counted frames!");
        }
    }
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
