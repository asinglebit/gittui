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
        let current_branch_name = match get_current_branch(&self.repo) {
            Some(branch) => format!(" ● {}", branch),
            None => format!(" ○ HEAD: {}", self.repo.head().unwrap().target().unwrap()),
        };

        let sha_paragraph = ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![
            Span::styled(" GUITAR |", Style::default().fg(COLOR_TEXT)),
            Span::styled(current_branch_name, Style::default().fg(COLOR_TEXT)),
        ])))
        .left_aligned()
        .block(Block::default());
        frame.render_widget(sha_paragraph, self.layout.title_left);
    }
}