#[rustfmt::skip]
use git2::{
    Oid
};
use ratatui::text::Line;
#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Span,
        Text
    },
    widgets::{
        Block,
        Borders,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        Wrap,
    },
};
#[rustfmt::skip]
use crate::{
    git::{
        queries::{
            get_uncommitted_changes,
            get_changed_filenames,
            FileStatus
        },
    },
    utils::{
        colors::*,
        symbols::*
    },
    app::app::{
        App,
        Panes
    }
};
#[rustfmt::skip]

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
        
        // Text
        let mut status_top_text: Text = Text::from("");
        let mut is_staged_changes = false;
        let mut is_unstaged_changes = false;
        let mut status_bottom_text: Text = Text::from("Hello world");

        // If viewing uncommitted changes
        if self.selected != 0 {
            let sha: Oid = *self.oids.get(self.selected).unwrap();
            let files = get_changed_filenames(&self.repo, sha);
            let mut lines = Vec::new();
            for file_change in files {
                let (symbol, color) = match file_change.status {
                    FileStatus::Added => ("+ ", COLOR_GREEN),
                    FileStatus::Modified => ("~ ", COLOR_BLUE),
                    FileStatus::Deleted => ("- ", COLOR_RED),
                    FileStatus::Renamed => ("→ ", COLOR_YELLOW),
                    FileStatus::Other => ("  ", COLOR_TEXT),
                };
                let display_filename = truncate_with_ellipsis(&file_change.filename, max_text_width);
                lines.push(Line::from(vec![
                    Span::styled(symbol, Style::default().fg(color)),
                    Span::styled(display_filename, Style::default().fg(COLOR_TEXT)),
                ]));
            }
            if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No changes",
                    Style::default().fg(COLOR_GREY_400),
                )));
            }
            status_top_text = Text::from(lines)
        } else {
            let changes = get_uncommitted_changes(&self.repo).unwrap();
            
            // Staged changes with prefix
            let mut lines = Vec::new();
            for file in changes.staged.modified.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("~ ", Style::default().fg(COLOR_BLUE)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in changes.staged.added.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("+ ", Style::default().fg(COLOR_GREEN)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in changes.staged.deleted.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("- ", Style::default().fg(COLOR_RED)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            is_staged_changes = true;
            if lines.is_empty() {
                is_staged_changes = false;
                let visible_height = self.layout.status_top.height as usize;
                let blank_lines_before = visible_height.saturating_sub(3) / 2;
                let mut display_lines = Vec::new();
                for _ in 0..blank_lines_before {
                    display_lines.push(Line::from(""));
                }
                display_lines.push(Line::from(Span::styled(
                    truncate_with_ellipsis("⊘ no staged changes", max_text_width),
                    Style::default().fg(COLOR_GREY_800),
                )));
                lines = display_lines;
            }
            status_top_text = Text::from(lines);
            
            // Unstaged changes with prefix
            let mut lines = Vec::new();
            for file in changes.unstaged.modified.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("~ ", Style::default().fg(COLOR_BLUE)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in changes.unstaged.added.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("+ ", Style::default().fg(COLOR_GREEN)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            for file in changes.unstaged.deleted.into_iter() {
                lines.push(Line::from(vec![
                    Span::styled("- ", Style::default().fg(COLOR_RED)),
                    Span::styled(truncate_with_ellipsis(&file, max_text_width), Style::default().fg(COLOR_TEXT)),
                ]));
            }
            is_unstaged_changes = true;
            if lines.is_empty() {
                is_unstaged_changes = false;
                if lines.is_empty() {
                    is_staged_changes = false;
                    let visible_height = self.layout.status_bottom.height as usize;
                    let blank_lines_before = visible_height.saturating_sub(2) / 2;
                    let mut display_lines = Vec::new();
                    for _ in 0..blank_lines_before {
                        display_lines.push(Line::from(""));
                    }
                    display_lines.push(Line::from(Span::styled(
                        truncate_with_ellipsis("⊘ no unstaged changes", max_text_width),
                        Style::default().fg(COLOR_GREY_800),
                    )));
                    lines = display_lines;
                }
            }
            status_bottom_text = Text::from(lines);
        }
        
        let total_file_lines = status_top_text.lines.len();
        let visible_height = self.layout.status_top.height as usize;
        let status_paragraph = ratatui::widgets::Paragraph::new(status_top_text)
            .alignment(if !is_staged_changes && self.selected == 0 {ratatui::layout::Alignment::Center} else {ratatui::layout::Alignment::Left})
            .wrap(Wrap { trim: false })
            .scroll((self.status_scroll.get() as u16, 0))
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(if self.selected == 0 { " (s)taged " } else { " (s)tatus " }, Style::default().fg(if self.focus == Panes::StatusTop { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_bottom(if self.selected == 0 {vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" unstaged ", Style::default().fg(if self.focus == Panes::StatusBottom { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ]} else {vec![]})
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .padding(padding)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        frame.render_widget(status_paragraph, self.layout.status_top);

        // Render the scrollbar
        let mut scrollbar_state =
            ScrollbarState::new(total_file_lines).position(self.status_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(if self.is_inspector && self.selected != 0 { Some("│") } else { Some("╮") })
            .end_symbol(if self.selected == 0 { Some("┤") } else {  Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol(if total_file_lines > visible_height {
                "▌"
            } else {
                "│"
            })
            .thumb_style(Style::default().fg(if total_file_lines > visible_height {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        frame.render_stateful_widget(scrollbar, self.layout.status_top, &mut scrollbar_state);

        if self.selected == 0 {
            let total_file_lines = status_bottom_text.lines.len();
            let visible_height = self.layout.status_bottom.height as usize;
            let status_paragraph = ratatui::widgets::Paragraph::new(status_bottom_text)
                .alignment(if !is_unstaged_changes && self.selected == 0 {ratatui::layout::Alignment::Center} else {ratatui::layout::Alignment::Left})
                .wrap(Wrap { trim: false })
                .scroll((self.status_scroll.get() as u16, 0))
                .block(
                    Block::default()
                        .borders(Borders::BOTTOM | Borders::RIGHT)
                        .border_style(Style::default().fg(COLOR_BORDER))
                        .padding(padding)
                        .border_type(ratatui::widgets::BorderType::Rounded),
                );

            frame.render_widget(status_paragraph, self.layout.status_bottom);

            // Render the scrollbar
            let mut scrollbar_state =
                ScrollbarState::new(total_file_lines).position(self.status_scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("│"))
                .end_symbol(Some("╯"))
                .track_symbol(Some("│"))
                .thumb_symbol(if total_file_lines > visible_height {
                    "▌"
                } else {
                    "│"
                })
                .thumb_style(Style::default().fg(if total_file_lines > visible_height {
                    COLOR_GREY_600
                } else {
                    COLOR_BORDER
                }));

            frame.render_stateful_widget(scrollbar, self.layout.status_bottom, &mut scrollbar_state);
        }
    }
}
