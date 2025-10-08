#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Line,
        Span,
        Text
    },
    widgets::{
        Block,
        Clear
    },
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*,
    },
};
#[rustfmt::skip]
use crate::app::{
    app::App,
};

impl App {

    pub fn draw_status(&mut self, frame: &mut Frame) {
        let status_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![Span::styled(
                format!(" 🖿  {}", self.path),
                Style::default().fg(COLOR_TEXT),
            )])))
            .left_aligned()
            .block(Block::default());

        frame.render_widget(status_paragraph, self.layout.status_left);

        let title_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(
                format!("{}/{}", self.selected + 1, self.lines_messages.len()),
                Style::default().fg(COLOR_TEXT),
            ))))
            .right_aligned()
            .block(Block::default());

        frame.render_widget(Clear, self.layout.status_right);
        frame.render_widget(title_paragraph, self.layout.status_right);
    }
}