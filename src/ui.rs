use ratatui::layout::Constraint::{Fill, Percentage};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::block::Title;
use ratatui::widgets::Cell;
use ratatui::{prelude::*, widgets::*};
use ratatui::Frame;


use crate::frame::CapturedFrame;
use crate::App;

pub fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Percentage(5),
        Fill(90)
    ]).split(f.size());

    let keybindings = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q> ".blue().bold(),
        " Clear Frame Info ".into(),
        "<C> ".blue().bold(),
    ]));

    draw_captured_frames(f, app, rects[1], keybindings);

    let frame_info = app.frame_info.lock().unwrap();
    let n_unique_frames = frame_info.captured_frames.len();
    let n_total_frames = frame_info.total_frame_count;
    let frames_per_second = frame_info.frames_per_second;

    draw_header(
        f,
        rects[0],
        n_total_frames,
        n_unique_frames,
        frames_per_second,
    );
}

// TODO: Implement this to allow switching between Hex and Dec string representations
fn frame_data_as_str(_frame: &CapturedFrame, _app: &App) {}

// TODO: A lil ugly that this is the place responsible for drawing keybindings
fn draw_captured_frames(f: &mut Frame, app: &mut App, area: Rect, keybindings: Title) {
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
        cells.push(Cell::from(Text::from(frame.get_data_string().to_string())));
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
    .highlight_style(selected_style)
    .block(
        Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_set(border::THICK)
        .title(
            keybindings
              .alignment(Alignment::Center)
              .position(block::Position::Bottom)
        )
    );

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_header(
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
