#[rustfmt::skip]
#[rustfmt::skip]
use git2::{
    Oid
};
use ratatui::widgets::Wrap;
#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Line,
        Span,
        Text
    },
    widgets::{
        Block,
        Borders,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
    },
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*,
        symbols::truncate_with_ellipsis,
        time::timestamp_to_utc
    },
    app::app::{
        App,
        Panes
    },
};

impl App {

    pub fn draw_inspector(&mut self, frame: &mut Frame) {

        // Calculate maximum available width for text
        let available_width = self.layout.inspector.width as usize - 2;
        let max_text_width = available_width.saturating_sub(1);

        let mut commit_lines: Vec<Line<'_>> = Vec::new();
        let sha: Oid = *self.oids.get(self.selected).unwrap();
        if sha != Oid::zero() {
            let commit = self.repo.find_commit(sha).unwrap();
            let author = commit.author();
            let committer = commit.committer();
            let summary = commit.summary().unwrap_or("<no summary>").to_string();
            let body = commit.body().unwrap_or("<no body>").to_string();
            let branches = self.oid_branch_map.get(&sha).unwrap();

            commit_lines = vec![
                Line::from(vec![Span::styled(
                    "Commit sha:",
                    Style::default().fg(COLOR_GREY_400),
                )]),
                Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("{}", self.oids.get(self.selected).unwrap()), max_text_width),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    "Parent shas:",
                    Style::default().fg(COLOR_GREY_400),
                )]),
            ];

            for parent_id in commit.parent_ids() {
                commit_lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("{}", parent_id), max_text_width),
                    Style::default().fg(COLOR_TEXT),
                )]));
            }

            commit_lines.extend(vec![
                Line::from(vec![Span::styled(
                    format!("Featured branches:"),
                    Style::default().fg(COLOR_GREY_400),
                )]),
            ]);

            for branch in branches {
                let oid = self.branch_oid_map.get(branch).unwrap();
                let color = self.tip_colors.get(oid).unwrap();
                commit_lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("● {}", branch), max_text_width),
                    Style::default().fg(*color),
                )]));
            }

            commit_lines.extend(vec![
                Line::from(vec![Span::styled(
                    format!("Authored by: {}", author.name().unwrap_or("-")),
                    Style::default().fg(COLOR_GREY_400),
                )]),
                Line::from(vec![Span::styled(
                    author.email().unwrap_or("").to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(author.when()),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    format!("Commited by: {}", committer.name().unwrap_or("-")),
                    Style::default().fg(COLOR_GREY_400),
                )]),
                Line::from(vec![Span::styled(
                    committer.email().unwrap_or("").to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(committer.when()).to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![
                    Span::styled("Message summary: ", Style::default().fg(COLOR_GREY_400)),
                    Span::styled(summary, Style::default().fg(COLOR_TEXT)),
                ]),
                Line::from(vec![
                    Span::styled("Message body: ", Style::default().fg(COLOR_GREY_400)),
                    Span::styled(body, Style::default().fg(COLOR_TEXT)),
                ]),
            ]);
        }

        let visible_height = self.layout.inspector.height as usize;
        let total_inspector_lines = commit_lines
            .iter()
            .map(|line| {
                let line_str: String = line
                    .spans
                    .iter()
                    .map(|span| span.content.trim())
                    .collect::<Vec<_>>()
                    .join("");
                let visual_width = line_str.len(); // approximate: counts chars, may differ for wide unicode
                // How many wrapped lines this line takes
                let wrapped_lines = (visual_width + self.layout.inspector.width as usize)
                    / self.layout.inspector.width as usize;
                wrapped_lines.max(1) // at least 1 line
            })
            .sum::<usize>();
        
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };

        let commit_paragraph = ratatui::widgets::Paragraph::new(Text::from(commit_lines))
            .left_aligned()
            .wrap(Wrap { trim: true }) //For some reasone causes ghosting
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" (i)nspector ", Style::default().fg(if self.focus == Panes::Inspector { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(if self.is_status { Borders::RIGHT | Borders::TOP } else { Borders::RIGHT | Borders::TOP | Borders::BOTTOM })
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .padding(padding)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        frame.render_widget(commit_paragraph, self.layout.inspector);

        // Render the scrollbar
        let mut scrollbar_state =
            ScrollbarState::new(total_inspector_lines).position(self.status_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(if self.is_status { Some("│") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol(if total_inspector_lines > visible_height {
                "▌"
            } else {
                "│"
            })
            .thumb_style(
                Style::default().fg(if total_inspector_lines > visible_height {
                    COLOR_GREY_600
                } else {
                    COLOR_BORDER
                }),
            );

        frame.render_stateful_widget(scrollbar, self.layout.inspector, &mut scrollbar_state);
    }
}