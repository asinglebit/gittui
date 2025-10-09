#[rustfmt::skip]
use ratatui::{
    Frame,
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Layout
};

impl App {

    pub fn layout(&mut self, frame: &mut Frame) {

        let is_inspector = self.is_inspector && self.selected != 0;
        let is_pane_visible = is_inspector || self.is_status;

        let chunks_vertical = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(if self.is_minimal { 0 } else { 1 }),
                ratatui::layout::Constraint::Percentage(100),
                ratatui::layout::Constraint::Length(if self.is_minimal { 0 } else { 1 }),
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
                ratatui::layout::Constraint::Percentage(if is_pane_visible { 70 } else { 100 }),
                ratatui::layout::Constraint::Percentage(if is_pane_visible { 30 } else { 0 }),
            ])
            .split(chunks_vertical[1]);

        let chunks_pane = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(if is_inspector { if !self.is_status { 100 } else { 30 } } else { 0 }),
                ratatui::layout::Constraint::Percentage(if self.is_status { if !is_inspector { 100 } else { 70 } } else { 0 }),
            ])
            .split(chunks_horizontal[1]);

        let chunks_status = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(if self.selected == 0 { 50 } else { 100 }),
                ratatui::layout::Constraint::Percentage(if self.selected == 0 { 50 } else { 0 }),
            ])
            .split(chunks_pane[1]);

        let chunks_status_bar = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(80),
                ratatui::layout::Constraint::Percentage(20),
            ])
            .split(chunks_vertical[2]);

        self.layout = Layout {
            title_left: chunks_title_bar[0],
            title_right: chunks_title_bar[1],
            graph: chunks_horizontal[0],
            inspector: chunks_pane[0],
            status_top: chunks_status[0],
            status_bottom: chunks_status[1],
            statusbar_left: chunks_status_bar[0],
            statusbar_right: chunks_status_bar[1]
        }
    }
}
