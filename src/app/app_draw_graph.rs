#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::Span,
    widgets::{
        Block,
        Borders,
        Cell as WidgetCell,
        Clear,
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
    Panes
};

impl App {

    pub fn draw_graph(&mut self, frame: &mut Frame) {

        let table_height = self.layout.graph.height as usize - 2;
        let total_rows = self.lines_graph.len();

        // Make sure selected row is visible
        if self.selected < self.scroll.get() {
            self.scroll.set(self.selected);
        } else if self.selected >= self.scroll.get() + table_height {
            self.scroll
                .set(self.selected.saturating_sub(table_height - 1));
        }

        let start = self.scroll.get();
        let end = (self.scroll.get() + table_height).min(total_rows);

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
            let mut row = Row::new(vec![
                WidgetCell::from(graph.clone()),
                WidgetCell::from(branch.clone()),
                WidgetCell::from(buffer.clone()),
            ]);

            if actual_index == self.selected {
                row = row.style(Style::default().bg(COLOR_GREY_800).fg(COLOR_GREY_600));
            }
            rows.push(row);
        }

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(25),
                ratatui::layout::Constraint::Percentage(100),
            ],
        )
        .block(
            Block::default()
                .title(vec![
                    Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    Span::styled(" (g)raph ", Style::default().fg(if self.focus == Panes::Graph { COLOR_GREY_500 } else { COLOR_TEXT } )),
                    Span::styled("─", Style::default().fg(COLOR_BORDER)),
                ])
                .title_alignment(ratatui::layout::Alignment::Right)
                .title_style(Style::default().fg(COLOR_GREY_400))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .row_highlight_style(Style::default().bg(COLOR_SELECTION).fg(COLOR_TEXT_SELECTED))
        .column_spacing(2);

        frame.render_widget(Clear, self.layout.graph);

        frame.render_widget(table, self.layout.graph);

        // Render the scrollbar
        let total_lines = self.oids.len();
        let visible_height = self.layout.graph.height as usize;
        if total_lines > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_lines).position(self.scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╮") })
                .end_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╯") })
                .track_symbol(Some("│"))
                .thumb_symbol("▌")
                .thumb_style(Style::default().fg(COLOR_GREY_600));

            frame.render_stateful_widget(scrollbar, self.layout.graph, &mut scrollbar_state);
        }
    }
}
