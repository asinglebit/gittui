#[rustfmt::skip]
use ratatui::{
    layout::Rect,
    Frame
};

pub struct Layout {
    pub title_left: Rect,
    pub title_right: Rect,
    pub graph: Rect,
    pub inspector: Rect,
    pub files: Rect,
    pub status_left: Rect,
    pub status_right: Rect
}

pub fn generate_layout(frame: &Frame, is_minimal: bool) -> Layout {
    let chunks_vertical = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(if is_minimal { 0 } else { 1 }),
            ratatui::layout::Constraint::Percentage(100),
            ratatui::layout::Constraint::Length(if is_minimal { 0 } else { 1 }),
        ])
        .split(frame.area());

    let chunks_title_bar = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(80),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .split(chunks_vertical[0]);

    let chunks_horizontal = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(70),
            ratatui::layout::Constraint::Percentage(30),
        ])
        .split(chunks_vertical[1]);

    let chunks_inspector = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage(40),
            ratatui::layout::Constraint::Percentage(60),
        ])
        .split(chunks_horizontal[1]);

    let chunks_status_bar = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage(80),
            ratatui::layout::Constraint::Percentage(20),
        ])
        .split(chunks_vertical[2]);

    Layout {
        title_left: chunks_title_bar[0],
        title_right: chunks_title_bar[1],
        graph: chunks_horizontal[0],
        inspector: chunks_inspector[0],
        files: chunks_inspector[1],
        status_left: chunks_status_bar[0],
        status_right: chunks_status_bar[1]
    }
}
