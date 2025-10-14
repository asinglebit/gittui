#[rustfmt::skip]
use std::cell::Cell;
#[rustfmt::skip]
use ratatui::{
    Frame,
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Layout,
    Viewport
};

impl App {

    pub fn layout(&mut self, frame: &mut Frame) {

        let is_settings = self.viewport == Viewport::Settings;
        let is_inspector = !is_settings && self.is_inspector && self.graph_selected != 0;
        let is_status = !is_settings && self.is_status;
        let is_right_pane = is_inspector || is_status;

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
                ratatui::layout::Constraint::Percentage(if is_right_pane { 70 } else { 100 }),
                ratatui::layout::Constraint::Percentage(if is_right_pane { 30 } else { 0 }),
            ])
            .split(chunks_vertical[1]);

        let chunks_pane = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(if is_inspector { if !is_status { 100 } else { 30 } } else { 0 }),
                ratatui::layout::Constraint::Percentage(if is_status { if !is_inspector { 100 } else { 70 } } else { 0 }),
            ])
            .split(chunks_horizontal[1]);

        let chunks_status = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(if self.graph_selected == 0 { 50 } else { 100 }),
                ratatui::layout::Constraint::Percentage(if self.graph_selected == 0 { 50 } else { 0 }),
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

    pub fn trap_selection(&self, selected: usize, scroll: &Cell<usize>, total_lines: usize, visible_height: usize) {
        if visible_height == 0 || total_lines == 0 {
            scroll.set(0);
            return;
        }

        // Max scroll offset so that a full page fits (if total_lines < visible_height, max_scroll = 0)
        let max_scroll = total_lines.saturating_sub(visible_height);

        // Get current scroll and clamp it to max_scroll
        let mut scroll_val = scroll.get().min(max_scroll);
        let sel = selected.min(total_lines.saturating_sub(1));

        // If selection is above the viewport -> jump scroll up
        if sel < scroll_val {
            scroll_val = sel;
            scroll.set(scroll_val);
            return;
        }

        // If selection is below the viewport -> jump scroll down so selection is the last visible line
        if sel >= scroll_val + visible_height {
            let desired = sel.saturating_sub(visible_height).saturating_add(1);
            scroll_val = desired.min(max_scroll);
            scroll.set(scroll_val);
            return;
        }

        // Otherwise selection is already visible; ensure scroll is clamped
        scroll.set(scroll_val);
    }
}
