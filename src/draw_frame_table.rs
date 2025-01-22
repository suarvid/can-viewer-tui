use embedded_can::Frame;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Cell, Row, Table},
};
use socketcan::CanFrame;

use crate::{frame::TimestampedFrame, App};

fn get_header_for_timestamped_frames(header_style: Style) -> Row<'static> {
    [
        "Frame #",
        "Timestamp",
        "ID",
        "DLC",
        "Extended",
        "Data (hex)",
    ]
    .into_iter()
    .map(Cell::from)
    .collect::<Row>()
    .style(header_style)
}

fn get_row_for_timestamped_frame<'a>(frame: &TimestampedFrame) -> Vec<Cell<'a>> {
    let mut cells = vec![];
    cells.push(Cell::from(Text::from(format!("{}", frame.frame_number))));
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
        "{:x?}",
        CanFrame::data(&frame.frame)
    ))));

    cells
}

fn draw_timestamped_frames(
    rows: Vec<Row<'_>>,
    header_style: Style,
    selected_style: Style,
    f: &mut ratatui::Frame<'_>,
    area: Rect,
    app: &mut App<'_>,
) {
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(100),
        ],
    )
    .header(get_header_for_timestamped_frames(header_style))
    .highlight_style(selected_style);
    f.render_stateful_widget(table, area, &mut app.table_state);
}

pub fn draw_captured_frames(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let header_style = Style::default().fg(Color::White).bg(Color::Black);
    let selected_style = Style::default().fg(Color::Black).bg(Color::LightYellow);

    let mut rows: Vec<Row> = Vec::new();

    match &app
        .frame_captor
        .get_captured_frames()
        .lock()
        .unwrap()
        .captured_frames
    {
        crate::frame::CapturedFrames::List(vec) => {
            if let Some(filter) = &app.frame_id_filter {
                vec.iter()
                    .rev()
                    .filter(|f| (filter.filter_callback)(f, &filter.ids))
                    .take(100)
                    .enumerate()
                    .for_each(|(i, frame)| {
                        let color = match i % 2 {
                            0 => app.row_color_main,
                            _ => app.row_color_alt,
                        };

                        let cells = get_row_for_timestamped_frame(&frame);
                        rows.push(
                            Row::new(cells).style(Style::default().fg(Color::Black).bg(color)),
                        );
                    });
            } else {
                vec.iter()
                    .rev()
                    .take(100)
                    .enumerate()
                    .for_each(|(i, frame)| {
                        let color = match i % 2 {
                            0 => app.row_color_main,
                            _ => app.row_color_alt,
                        };

                        let cells = get_row_for_timestamped_frame(&frame);
                        rows.push(
                            Row::new(cells).style(Style::default().fg(Color::Black).bg(color)),
                        );
                    });
            }

            draw_timestamped_frames(rows, header_style, selected_style, f, area, app);
        }
        crate::frame::CapturedFrames::Set(_hash_map) => {
            todo!("Add support for drawing set of counted frames!");
        }
    }
}
