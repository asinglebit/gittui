use ratatui::text::Line;
#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::Span,
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
        self.trap_selection(self.graph_selected, &self.graph_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.graph_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // Start with fake commit row
        let mut rows = Vec::with_capacity(end - start + 1); // preallocate for efficiency

        // Add the rest of the commits
        for (i, ((graph, branch), buffer)) in self.lines_graph[start..end]
            .iter()
            .zip(&self.lines_branches[start..end])
            .zip(&self.lines_buffers[start..end])
            .enumerate()
        {
            let actual_index = start + i;
            let (graph, branch, buffer) = if actual_index == self.graph_selected {
                let graph_spans: Vec<Span> = graph.spans.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(COLOR_GREY_400)) }).collect();
                let branch_spans: Vec<Span> = branch.spans.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(COLOR_GREY_400)) }).collect();
                let buffer_spans: Vec<Span> = buffer.spans.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(COLOR_GREY_400)) }).collect();
                (Line::from(graph_spans), Line::from(branch_spans), Line::from(buffer_spans))
            } else {
                (graph.clone(), branch.clone(), buffer.clone())
            };
            let mut row = Row::new(vec![
                WidgetCell::from(graph.clone()),
                WidgetCell::from(branch.clone()),
                WidgetCell::from(buffer.clone()),
            ]);
            if actual_index == self.graph_selected {
                row = row.style(Style::default().bg(COLOR_GREY_800));
            }
            rows.push(row);
        }

        // Setup the table
        let table = Table::new(rows, [
                ratatui::layout::Constraint::Length(25),
                ratatui::layout::Constraint::Percentage(70),
            ]).block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" graph ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .column_spacing(5);

        // Render the table
        frame.render_widget(table, self.layout.graph);

        if total_lines > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.graph_scroll.get());
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
}
