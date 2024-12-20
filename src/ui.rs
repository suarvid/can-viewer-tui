use embedded_can::Frame;
use ratatui::layout::Constraint::Percentage;
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
        .constraints([Percentage(5), Percentage(70), Percentage(25)])
        .split(f.area());

    let keybindings = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
        " Clear Frame Info ".into(),
        "<C> ".blue().bold(),
        " Go To Latest Frame".into(),
        "<G> ".blue().bold(),
    ]));

    draw_captured_frames(f, app, rects[1]);

    draw_frames_per_second_chart(
        f,
        rects[2],
        app.frame_captor.get_frames_per_second_history(),
        keybindings,
        app.frames_per_second_max,
    );

    let n_unique_frames = app.frame_captor.get_unique_frame_count();

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
        "{:?}",
        CanFrame::data(&frame.frame)
    ))));

    cells
}

fn get_header_for_timestamped_frames(header_style: Style) -> Row<'static> {
    ["Frame #", "Timestamp", "ID", "DLC", "Extended", "Data"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
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
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Percentage(25),
        ],
    )
    .header(get_header_for_timestamped_frames(header_style))
    .highlight_style(selected_style);
    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_frames_per_second_chart(
    frame: &mut ratatui::Frame,
    area: Rect,
    data: Vec<(f64, f64)>,
    keybindings: Title<'_>,
    frames_per_second_max: u32,
) {
    let x_limit_lo = 0.0;
    let x_limit_hi = 300.0;

    let y_limit_lo = 0.0;
    let y_limit_hi = frames_per_second_max as f64;

    let dataset = vec![Dataset::default()
        .marker(symbols::Marker::Dot)
        .style(Style::default())
        .data(&data)];

    let x_labels = vec![
        Span::styled(format!("{}", x_limit_lo), Style::default()),
        Span::styled(format!("{}", x_limit_hi), Style::default()),
    ];

    let y_labels = vec![
        Span::styled(format!("{}", y_limit_lo), Style::default()),
        Span::styled(format!("{}", y_limit_hi), Style::default()),
    ];

    let chart = Chart::new(dataset)
        .block(Block::bordered())
        .x_axis(
            Axis::default()
                .title("Time (Seconds Ago)")
                .style(Style::default())
                .labels(x_labels)
                .bounds([x_limit_lo, x_limit_hi]),
        )
        .y_axis(
            Axis::default()
                .title("Frames per Second")
                .style(Style::default())
                .labels(y_labels)
                .bounds([y_limit_lo, y_limit_hi]),
        )
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

    frame.render_widget(chart, area);
}

fn draw_captured_frames(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let header_style = Style::default().fg(Color::White).bg(Color::Black);
    let selected_style = Style::default().fg(Color::Black).bg(Color::LightYellow);

    let rows = match app.frame_id_filter {
        Some(filter) => {
            let mut rows: Vec<Row> = Vec::new();
            app.frame_captor
                .get_captured_frames()
                .lock()
                .unwrap()
                .get_filtered_frames(filter)
                .iter()
                .enumerate()
                .for_each(|(i, frame)| {
                    let color = match i % 2 {
                        0 => app.row_color_main,
                        _ => app.row_color_alt,
                    };

                    let cells = get_row_for_timestamped_frame(&frame);
                    rows.push(Row::new(cells).style(Style::default().fg(Color::Black).bg(color)));
                });

            rows
        }
        None => {
            let mut rows: Vec<Row> = Vec::new();
            match &app
                .frame_captor
                .get_captured_frames()
                .lock()
                .unwrap()
                .captured_frames
            {
                crate::frame::CapturedFrames::List(vec) => {
                    vec.iter().rev().take(100).cloned().enumerate().for_each(|(i, frame)| {
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
                crate::frame::CapturedFrames::Set(_hash_map) => {
                    todo!("Add support for drawing set of counted frames!");
                }
            }

            rows
        }
    };

    draw_timestamped_frames(rows, header_style, selected_style, f, area, app);
}

fn draw_header(
    f: &mut ratatui::Frame,
    area: Rect,
    total_frame_count: usize,
    n_unique_frames: usize,
    frames_per_second: usize,
) {
    let header = Paragraph::new(Line::from(format!(
        "Unique Frame IDs: {}, Total Frame Count {}, Frames Per Second: {}",
        n_unique_frames, total_frame_count, frames_per_second
    )))
    .centered()
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double),
    );

    f.render_widget(header, area);
}
