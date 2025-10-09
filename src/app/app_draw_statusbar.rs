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
use crate::{
    app::app::{
        App,
    },
    git::queries::get_current_branch
};

impl App {

    pub fn draw_statusbar(&mut self, frame: &mut Frame) {
        
        let lines = match get_current_branch(&self.repo) {
            Some(branch) => Line::from(vec![Span::styled(format!(" ● {}", branch), Style::default().fg(COLOR_PURPLE))]),
            None => {
                let oid = self.repo.head().unwrap().target().unwrap();
                Line::from(vec![Span::styled(format!(" detached head: #{:.6}", oid), Style::default().fg(COLOR_TEXT))])
            },
        };
        let status_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(lines))
            .left_aligned()
            .block(Block::default());

        frame.render_widget(status_paragraph, self.layout.statusbar_left);

        let title_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(
                format!("{}/{}", self.graph_selected + 1, self.lines_messages.len()),
                Style::default().fg(COLOR_TEXT),
            ))))
            .right_aligned()
            .block(Block::default());

        frame.render_widget(Clear, self.layout.statusbar_right);
        frame.render_widget(title_paragraph, self.layout.statusbar_right);
    }
}