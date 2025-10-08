#[rustfmt::skip]
use git2::{
    Repository
};
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
use crate::{app::layout::layout::Layout, git::queries::get_current_branch};

pub fn render_title_bar(frame: &mut Frame, repo: &Repository, layout: &Layout) {
    let current_branch_name = match get_current_branch(repo) {
        Some(branch) => format!(" ● {}", branch),
        None => format!(" ○ HEAD: {}", repo.head().unwrap().target().unwrap()),
    };

    let sha_paragraph = ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![
        Span::styled(" GUITAR |", Style::default().fg(COLOR_TEXT)),
        Span::styled(current_branch_name, Style::default().fg(COLOR_TEXT)),
    ])))
    .left_aligned()
    .block(Block::default());
    frame.render_widget(sha_paragraph, layout.title_left);
}
