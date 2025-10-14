#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    widgets::{
        Block,
        Borders,
        Cell as WidgetCell,
        Row,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        Table,
    },
};
use crate::core::renderers::{render_buffer_range, render_graph_range, render_message_range};
#[rustfmt::skip]
use crate::{
    helpers::{
        palette::*,
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Focus
};

impl App {
    pub fn draw_graph(&mut self, frame: &mut Frame) {
        // Get vertical dimensions
        let total_lines = self.oids.len();
        let visible_height = self.layout.graph.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.graph_selected = 0;
        } else if self.graph_selected >= total_lines {
            self.graph_selected = total_lines - 1;
        }

        // Trap selection
        self.trap_selection(
            self.graph_selected,
            &self.graph_scroll,
            total_lines,
            visible_height,
        );

        // Calculate scroll
        let start = self
            .graph_scroll
            .get()
            .min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // History
        let history = self.buffer.borrow().history.clone();
        let head = self.repo.head().unwrap().target().unwrap();

        // Rendered lines
        let buffer_range = render_buffer_range(&history, start, end + 1);
        let graph_range = render_graph_range(
            &self.oids,
            &self.tips,
            &mut self.tip_colors,
            &history,
            head,
            start,
            end,
        );
        let message_range = render_message_range(
            &self.repo,
            &self.oids,
            &self.tips,
            &mut self.tip_colors,
            &history,
            start,
            end,
            self.graph_selected,
            &self.uncommitted,
        );

        // Start with fake commit row
        let mut rows = Vec::with_capacity(end - start + 1); // preallocate for efficiency

        // Add the rest of the commits
        let mut width = 0;

        if !graph_range.is_empty() {
            for idx in 0..graph_range.len() {
                width = graph_range
                    .iter()
                    .map(|line| {
                        line.spans
                            .iter()
                            .filter(|span| !span.content.is_empty()) // only non-empty spans
                            .map(|span| span.content.chars().count()) // use chars() for wide characters
                            .sum::<usize>()
                    })
                    .max()
                    .unwrap_or(0) as u16;

                let mut row = Row::new(vec![
                    WidgetCell::from(graph_range.get(idx).cloned().unwrap_or_default()),
                    WidgetCell::from(message_range.get(idx).cloned().unwrap_or_default()),
                ]);
                if idx + start == self.graph_selected {
                    row = row.style(Style::default().bg(COLOR_GREY_800));
                }
                rows.push(row);
            }
        }

        // Setup the table
        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(width),
                ratatui::layout::Constraint::Min(0),
            ],
        )
        .block(
            Block::default()
                // .title(vec![
                //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                //     Span::styled(" graph ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                // ])
                // .title_alignment(ratatui::layout::Alignment::Right)
                // .title_style(Style::default().fg(COLOR_GREY_400))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .column_spacing(5);

        // Render the table
        frame.render_widget(table, self.layout.graph);

        if total_lines > visible_height {
            let mut scrollbar_state =
                ScrollbarState::new(total_lines.saturating_sub(visible_height))
                    .position(self.graph_scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(
                    if (self.is_inspector && self.graph_selected != 0) || self.is_status {
                        Some("─")
                    } else {
                        Some("╮")
                    },
                )
                .end_symbol(
                    if (self.is_inspector && self.graph_selected != 0) || self.is_status {
                        Some("─")
                    } else {
                        Some("╯")
                    },
                )
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
}
