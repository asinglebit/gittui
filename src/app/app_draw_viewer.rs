#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::Span,
    widgets::{
        Block,
        Borders,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        List,
        ListItem
    }
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Focus
};

impl App {

    pub fn draw_viewer(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.graph.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        // Get vertical dimensions
        let total_lines = self.viewer_lines.len();
        let visible_height = self.layout.graph.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.viewer_selected = 0;
        } else if self.viewer_selected >= total_lines {
            self.viewer_selected = total_lines - 1;
        }
        
        // Trap selection
        self.trap_selection(self.viewer_selected, &self.viewer_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.viewer_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = self.viewer_lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let absolute_idx = start + i;
                let mut item = line.clone();
                if absolute_idx == self.viewer_selected && self.focus == Focus::Viewport {
                    item = item.style(Style::default().bg(COLOR_GREY_800));
                }
                item
            })
            .collect();

        // Setup the list
        let list = List::new(list_items)
            .block(
                Block::default()
                    .padding(padding)
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" (v)iewer ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        // Render the list
        frame.render_widget(list, self.layout.graph);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.viewer_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╮") })
            .end_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol("▌")
            .thumb_style(Style::default().fg(if self.focus == Focus::Viewport {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.graph, &mut scrollbar_state);
    }
}
