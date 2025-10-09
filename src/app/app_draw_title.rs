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
use crate::{git::queries::get_current_branch};

impl App {

    pub fn draw_title(&mut self, frame: &mut Frame) {
        let span_current_checkout = match get_current_branch(&self.repo) {
            Some(branch) => Span::styled(format!(" ● {}", branch), Style::default().fg(COLOR_PURPLE)),
            None => Span::styled(format!(" ○ HEAD: {}", self.repo.head().unwrap().target().unwrap()), Style::default().fg(COLOR_TEXT)),
        };

        let sha_paragraph = ratatui::widgets::Paragraph::new(Text::from(Line::from([
            self.logo.clone(), vec![

            Span::styled(" |", Style::default().fg(COLOR_TEXT)),
            span_current_checkout,
        ]].concat())))
        .left_aligned()
        .block(Block::default());
        frame.render_widget(sha_paragraph, self.layout.title_left);
    }
}