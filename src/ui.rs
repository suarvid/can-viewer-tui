use ratatui::layout::Constraint::{Fill, Percentage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::Cell;
use ratatui::{prelude::*, widgets::*};
use ratatui::Frame;
use std::collections::HashMap;

use crate::frame::CapturedFrame;
use crate::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([Percentage(95), Fill(1)]).split(f.size());

    draw_captured_frames(f, app, rects[0]);
    let frame_info = app.frame_info.lock().unwrap();
    let n_unique_frames = frame_info.captured_frames.len();
    let n_total_frames = frame_info.total_frame_count;
    let frames_per_second = frame_info.frames_per_second;

    draw_footer(
        f,
        rects[1],
        n_total_frames,
        n_unique_frames,
        frames_per_second,
    );
}

// Kan ju kolla typ data_rep_mode i app, ifall format-strängen ska vara
// decimal eller hex
fn frame_data_as_str(frame: &CapturedFrame, app: &App) {}

fn draw_captured_frames(f: &mut Frame, app: &mut App, area: Rect) {
    let frame_info = app.frame_info.lock().unwrap();
    let header_style = Style::default().fg(Color::White).bg(Color::Black);

    let selected_style = Style::default().fg(Color::Black).bg(Color::LightYellow);

    let header = ["ID", "DLC", "Count", "Extended", "Data"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let mut rows: Vec<Row> = Vec::new();

    for (i, (_, frame)) in frame_info.captured_frames.iter().enumerate() {
        let color = match i % 2 {
            0 => app.row_color_main,
            _ => app.row_color_alt,
        };

        let mut cells = vec![];
        cells.push(Cell::from(Text::from(format!("{:#01x}", frame.id))));
        cells.push(Cell::from(Text::from(format!("{}", frame.dlc))));
        cells.push(Cell::from(Text::from(format!("{}", frame.count))));
        cells.push(Cell::from(Text::from(format!("{}", frame.is_extended))));
        cells.push(Cell::from(Text::from(format!(
            "{}",
            frame.get_data_string()
        ))));
        rows.push(
            Row::new(cells)
                .style(Style::default().fg(Color::Black).bg(color))
                .height(1),
        );
    }

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
    .header(header)
    .highlight_style(selected_style);

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_footer(
    f: &mut Frame,
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
