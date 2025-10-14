#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Span,
        Line
    },
    widgets::{
        Block,
        Borders,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        List,
        ListItem
    },
};
#[rustfmt::skip]
use crate::{
    git::{
        queries::{
            helpers::{
                FileStatus
            }
        }
    },
    helpers::{
        palette::*,
        text::*
    },
    app::app::{
        App,
        Focus
    }
};

impl App {

    pub fn draw_status(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.status_top.width as usize - 3;
        let max_text_width = available_width.saturating_sub(2);

        // Flags
        let mut is_staged_changes = false;
        let mut is_unstaged_changes = false;
        let is_showing_uncommitted = self.graph_selected == 0;
        
        // Lines
        let mut lines_status_top: Vec<Line<'_>> = Vec::new();
        let mut lines_status_bottom: Vec<Line<'_>> = Vec::new();

        // If viewing uncommitted changes
        if is_showing_uncommitted {
            
            // Staged changes with prefix
            for file in self.uncommitted.staged.modified.iter() {
                lines_status_top.push(Line::from(vec![
                    Span::styled("~ ", Style::default().fg(COLOR_BLUE)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in self.uncommitted.staged.added.iter() {
                lines_status_top.push(Line::from(vec![
                    Span::styled("+ ", Style::default().fg(COLOR_GREEN)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in self.uncommitted.staged.deleted.iter() {
                lines_status_top.push(Line::from(vec![
                    Span::styled("- ", Style::default().fg(COLOR_RED)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            
            // Handle no changes
            if lines_status_top.is_empty() {
                let visible_height = self.layout.status_bottom.height as usize;
                let blank_lines_before = visible_height.saturating_sub(3) / 2;
                for _ in 0..blank_lines_before {
                    lines_status_top.push(Line::from(""));
                }
                lines_status_top.push(Line::from(Span::styled(
                    center_line(&truncate_with_ellipsis("⊘ no staged changes", max_text_width), max_text_width + 3),
                    Style::default().fg(COLOR_GREY_800),
                )));
            } else {
                is_staged_changes = true;
            }
            
            // Unstaged changes with prefix
            for file in self.uncommitted.unstaged.modified.iter() {
                lines_status_bottom.push(Line::from(vec![
                    Span::styled("~ ", Style::default().fg(COLOR_BLUE)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in self.uncommitted.unstaged.added.iter() {
                lines_status_bottom.push(Line::from(vec![
                    Span::styled("+ ", Style::default().fg(COLOR_GREEN)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in self.uncommitted.unstaged.deleted.iter() {
                lines_status_bottom.push(Line::from(vec![
                    Span::styled("- ", Style::default().fg(COLOR_RED)),
                    Span::styled(truncate_with_ellipsis(file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            
            // Handle no changes
            if lines_status_bottom.is_empty() {
                let visible_height = self.layout.status_top.height as usize;
                let blank_lines_before = visible_height.saturating_sub(2) / 2;
                for _ in 0..blank_lines_before {
                    lines_status_bottom.push(Line::from(""));
                }
                lines_status_bottom.push(Line::from(Span::styled(
                    center_line(&truncate_with_ellipsis("⊘ no unstaged changes", max_text_width), max_text_width + 3),
                    Style::default().fg(COLOR_GREY_800),
                )));
            } else {
                is_unstaged_changes = true;
            }
        } else {
            
            // Assemble lines
            for file_change in self.current_diff.iter() {
                let (symbol, color) = match file_change.status {
                    FileStatus::Added => ("+ ", COLOR_GREEN),
                    FileStatus::Modified => ("~ ", COLOR_BLUE),
                    FileStatus::Deleted => ("- ", COLOR_RED),
                    FileStatus::Renamed => ("→ ", COLOR_YELLOW),
                    FileStatus::Other => ("  ", COLOR_TEXT),
                };
                let display_filename = truncate_with_ellipsis(&file_change.filename, max_text_width);
                lines_status_top.push(Line::from(vec![
                    Span::styled(symbol, Style::default().fg(color)),
                    Span::styled(display_filename, Style::default().fg(COLOR_TEXT)),
                ]));
            }

            // Handle no changes
            if lines_status_top.is_empty() {
                let visible_height = self.layout.status_top.height as usize;
                let blank_lines_before = visible_height.saturating_sub(3) / 2;
                for _ in 0..blank_lines_before {
                    lines_status_top.push(Line::from(""));
                }
                lines_status_top.push(Line::from(Span::styled(
                    center_line(&truncate_with_ellipsis("⊘ no staged changes", max_text_width), max_text_width + 3),
                    Style::default().fg(COLOR_GREY_800),
                )));
            } else {
                is_staged_changes = true;
            }
        }
        
        // Render status top
        {
            // Get vertical dimensions
            let total_lines = lines_status_top.len();
            let visible_height = self.layout.status_top.height as usize - 2;

            // Clamp selection
            if total_lines == 0 {
                self.status_top_selected = 0;
            } else if self.status_top_selected >= total_lines {
                self.status_top_selected = total_lines - 1;
            }

            // Trap selection
            self.trap_selection(self.status_top_selected, &self.status_top_scroll, total_lines, visible_height);

            // Calculate scroll
            let start = self.status_top_scroll.get().min(total_lines.saturating_sub(visible_height));
            let end = (start + visible_height).min(total_lines);

            // Setup list items
            let list_items: Vec<ListItem> = lines_status_top[start..end]
                .iter()
                .enumerate()
                .map(|(i, line)| {
                    if is_staged_changes && start + i == self.status_top_selected && self.focus == Focus::StatusTop {
                        let spans: Vec<Span> = line.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(COLOR_GRASS)) }).collect();
                        ListItem::new(Line::from(spans)).style(Style::default().bg(COLOR_GREY_800).fg(COLOR_GREY_400))
                    } else {
                        ListItem::new(line.clone())
                    }
                })
                .collect();

            // Setup the list
            let list = List::new(list_items)
                .block(
                    Block::default()
                        .padding(padding)
                        // .title(vec![
                        //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        //     Span::styled(if self.graph_selected == 0 { " (s)taged " } else { " (s)tatus " }, Style::default().fg(if self.focus == Focus::StatusTop { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        // ])
                        // .title_bottom(if self.graph_selected == 0 {vec![
                        //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        //     Span::styled(" unstaged ", Style::default().fg(if self.focus == Focus::StatusBottom { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        // ]} else {vec![]})
                        // .title_alignment(Alignment::Right)
                        // .title_style(Style::default().fg(COLOR_GREY_500))
                        .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
                        .border_style(Style::default().fg(COLOR_BORDER))
                        .border_type(ratatui::widgets::BorderType::Rounded),
                );

            frame.render_widget(list, self.layout.status_top);

            // Setup the scrollbar
            let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.status_top_scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(if self.is_inspector && self.graph_selected != 0 { Some("│") } else { Some("╮") })
                .end_symbol(if self.graph_selected == 0 { Some("┤") } else {  Some("╯") })
                .track_symbol(Some("│"))
                .thumb_symbol(if total_lines > visible_height {
                    "▌"
                } else {
                    "│"
                })
                .thumb_style(Style::default().fg(if total_lines > visible_height && self.focus == Focus::StatusTop {
                    COLOR_GREY_600
                } else {
                    COLOR_BORDER
                }));

            // Render the scrollbar
            frame.render_stateful_widget(scrollbar, self.layout.status_top, &mut scrollbar_state);
        }

        // Render status bottom
        {
            if is_showing_uncommitted {
                // Get vertical dimensions
                let total_lines = lines_status_bottom.len();
                let visible_height = self.layout.status_bottom.height.saturating_sub(2) as usize;

                // Clamp selection
                if total_lines == 0 {
                    self.status_bottom_selected = 0;
                } else if self.status_bottom_selected >= total_lines {
                    self.status_bottom_selected = total_lines - 1;
                }

                // Trap selection
                self.trap_selection(self.status_bottom_selected, &self.status_bottom_scroll, total_lines, visible_height);
                
                // Calculate scroll
                let start = self.status_bottom_scroll.get().min(total_lines.saturating_sub(visible_height));
                let end = (start + visible_height).min(total_lines);

                // Setup list items
                let list_items: Vec<ListItem> = lines_status_bottom[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        if is_unstaged_changes && start + i == self.status_bottom_selected && self.focus == Focus::StatusBottom {
                            let spans: Vec<Span> = line.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(COLOR_GRASS)) }).collect();
                            ListItem::new(Line::from(spans)).style(Style::default().bg(COLOR_GREY_800).fg(COLOR_GREY_400))
                        } else {
                            ListItem::new(line.clone())
                        }
                    })
                    .collect();
        
                // Setup the list
                let list = List::new(list_items)
                    .block(
                        Block::default()
                            .padding(padding)
                            .borders(Borders::BOTTOM | Borders::RIGHT)
                            .border_style(Style::default().fg(COLOR_BORDER))
                            .border_type(ratatui::widgets::BorderType::Rounded),
                    );

                frame.render_widget(list, self.layout.status_bottom);

                // Setup the scrollbar
                let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.status_bottom_scroll.get());
                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("│"))
                    .end_symbol(Some("╯"))
                    .track_symbol(Some("│"))
                    .thumb_symbol(if total_lines > visible_height {
                        "▌"
                    } else {
                        "│"
                    })
                    .thumb_style(Style::default().fg(if total_lines > visible_height && self.focus == Focus::StatusBottom {
                        COLOR_GREY_600
                    } else {
                        COLOR_BORDER
                    }));

                // Render the scrollbar
                frame.render_stateful_widget(scrollbar, self.layout.status_bottom, &mut scrollbar_state);
            }
        }
    }
}
