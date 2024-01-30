use ratatui::{
    prelude::{Alignment, Frame},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::client::app::App;

pub fn render(app: &mut App, f: &mut Frame) {
    f.render_widget(
        Paragraph::new(format!(
            "
                Press `Esc`, `Ctrl-C`, or `q` to stop running.\n\
                Press `j` and `k` to increment and decrement the counter.\n\
                Counter: {}
            ",
            27720,
        ))
        .block(
            Block::default()
                .title("AutoArcana Magic Sim")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center),
        f.size(),
    )
}

