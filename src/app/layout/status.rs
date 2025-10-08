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
use crate::{app::layout::layout::Layout};

pub fn render_status_bar(frame: &mut Frame, layout: &Layout, selected: usize, lines_messages: &[Line<'static>], path: &String ) {
    let status_paragraph =
        ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![Span::styled(
            format!(" 🖿  {}", path),
            Style::default().fg(COLOR_TEXT),
        )])))
        .left_aligned()
        .block(Block::default());

    frame.render_widget(status_paragraph, layout.status_left);

    let title_paragraph =
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(
            format!("{}/{}", selected + 1, lines_messages.len()),
            Style::default().fg(COLOR_TEXT),
        ))))
        .right_aligned()
        .block(Block::default());

    frame.render_widget(Clear, layout.status_right);
    frame.render_widget(title_paragraph, layout.status_right);
}
