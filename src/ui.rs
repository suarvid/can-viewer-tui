use ratatui::layout::Constraint::Percentage;
use ratatui::layout::{Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};

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
        " To Top of Table ".into(),
        "<T> ".blue().bold(),
        " Toggle Table View ".into(),
        "<V> ".blue().bold(),
    ]));

    (app.draw_frame_table)(f, app, rects[1]);

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
