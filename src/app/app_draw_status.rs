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
use crate::{git::queries::{get_changed_filenames, FileStatus}, utils::symbols::truncate_with_ellipsis};
#[rustfmt::skip]
use crate::{
    git::{
        queries::{
            get_uncommitted_changes
        },
    },
    utils::{
        colors::*
    },
};
#[rustfmt::skip]
use crate::app::app::App;

impl App {

    pub fn draw_status(&mut self, frame: &mut Frame) {
        
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.files.width as usize - 3;
        let max_text_width = available_width.saturating_sub(2);
        
        
        let mut files_text: Text = Text::from("-");
        let sha: Oid = *self.oids.get(self.selected).unwrap();
        if sha != Oid::zero() {
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

            files_text = Text::from(lines)
        } else {
            let changes = get_uncommitted_changes(&self.repo).unwrap();
            let mut lines = Vec::new();

            lines.push(Line::from(vec![
                Span::styled("Unstaged:", Style::default().fg(COLOR_TEXT)),
            ]));
            lines.push(Line::from(""));

            // Unstaged changes with prefix
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

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Staged:", Style::default().fg(COLOR_TEXT)),
            ]));
            lines.push(Line::from(""));

            // Staged changes with prefix
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

            if lines.is_empty() {
                lines.push(Line::from("No uncommitted changes"));
            }

            files_text = Text::from(lines);
        }
        let total_file_lines = files_text.lines.len();
        let visible_height = self.layout.files.height as usize;
        let files_paragraph = ratatui::widgets::Paragraph::new(files_text)
            .left_aligned()
            .wrap(Wrap { trim: false })
            .scroll((self.status_scroll.get() as u16, 0))
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" (s)tatus ", Style::default().fg(COLOR_TEXT)),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .padding(padding)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        frame.render_widget(files_paragraph, self.layout.files);

        // Render the scrollbar
        let mut scrollbar_state =
            ScrollbarState::new(total_file_lines).position(self.status_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(if self.is_inspector { Some("│") } else { Some("╮") })
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

        frame.render_stateful_widget(scrollbar, self.layout.files, &mut scrollbar_state);
    }
}