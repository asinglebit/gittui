#[rustfmt::skip]
use git2::{
    Oid
};
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
            get_changed_filenames_as_text
        },
    },
    utils::{
        colors::*
    },
};
#[rustfmt::skip]
use crate::app::app::App;

impl App {

    pub fn draw_files(&mut self, frame: &mut Frame) {
        
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        let mut files_text: Text = Text::from("-");
        let sha: Oid = *self.oids.get(self.selected).unwrap();
        if sha != Oid::zero() {
            files_text = get_changed_filenames_as_text(&self.repo, sha);
        }
        let total_file_lines = files_text.lines.len();
        let visible_height = self.layout.files.height as usize;
        let files_paragraph = ratatui::widgets::Paragraph::new(files_text)
            .left_aligned()
            .wrap(Wrap { trim: false })
            .scroll((self.files_scroll.get() as u16, 0))
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" Files ", Style::default().fg(COLOR_TEXT)),
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
            ScrollbarState::new(total_file_lines).position(self.files_scroll.get());
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

        frame.render_stateful_widget(scrollbar, self.layout.files, &mut scrollbar_state);
    }
}