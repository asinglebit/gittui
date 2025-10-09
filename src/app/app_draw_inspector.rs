#[rustfmt::skip]
use git2::{
    Oid
};
#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Line,
        Span,
    },
    layout::{
        Alignment,
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
    utils::{
        colors::*,
        symbols::{
            truncate_with_ellipsis,
            clean_commit_text
        },
        time::timestamp_to_utc
    },
    app::app::{
        App,
        Focus
    },
};

impl App {

    fn ensure_inspector_selected_visible(&self, total_lines: usize, visible_height: usize) {
        if visible_height == 0 || total_lines == 0 {
            self.inspector_scroll.set(0);
            return;
        }

        // Max scroll offset so that a full page fits (if total_lines < visible_height, max_scroll = 0)
        let max_scroll = total_lines.saturating_sub(visible_height);

        // Get current scroll and clamp it to max_scroll
        let mut scroll = self.inspector_scroll.get().min(max_scroll);
        let sel = self.inspector_selected.min(total_lines.saturating_sub(1));

        // If selection is above the viewport -> jump scroll up
        if sel < scroll {
            scroll = sel;
            self.inspector_scroll.set(scroll);
            return;
        }

        // If selection is below the viewport -> jump scroll down so selection is the last visible line
        if sel >= scroll + visible_height {
            let desired = sel.saturating_sub(visible_height).saturating_add(1);
            scroll = desired.min(max_scroll);
            self.inspector_scroll.set(scroll);
            return;
        }

        // Otherwise selection is already visible; ensure scroll is clamped
        self.inspector_scroll.set(scroll);
    }

    pub fn draw_inspector(&mut self, frame: &mut Frame) {
        
        // Calculate available width
        let available_width = self.layout.inspector.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        let mut commit_lines: Vec<Line<'_>> = Vec::new();

        if self.graph_selected != 0 {
            let sha: Oid = *self.oids.get(self.graph_selected).unwrap();
            let commit = self.repo.find_commit(sha).unwrap();
            let author = commit.author();
            let committer = commit.committer();
            let summary = commit.summary().unwrap_or("<no summary>").to_string();
            let body = commit.body().unwrap_or("<no body>").to_string();
            let branches = self.oid_branch_map.get(&sha).unwrap();

            commit_lines = vec![
                Line::from(vec![Span::styled("commit sha:", Style::default().fg(COLOR_GREY_500))]),
                Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("{}", sha), max_text_width),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled("parent shas:", Style::default().fg(COLOR_GREY_500))]),
            ];

            for parent_id in commit.parent_ids() {
                commit_lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("{}", parent_id), max_text_width),
                    Style::default().fg(COLOR_TEXT),
                )]));
            }

            commit_lines.extend(vec![
                Line::from(""),
            ]);

            commit_lines.push(Line::from(vec![Span::styled(
                "featured branches:",
                Style::default().fg(COLOR_GREY_500),
            )]));

            for branch in branches {
                let oid = self.branch_oid_map.get(branch).unwrap();
                let color = self.tip_colors.get(oid).unwrap();
                commit_lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("● {}", branch), max_text_width),
                    Style::default().fg(*color),
                )]));
            }

            commit_lines.extend(vec![
                Line::from(""),
            ]);

            commit_lines.extend(vec![
                Line::from(vec![Span::styled(
                    format!("authored by: {}", author.name().unwrap_or("-")),
                    Style::default().fg(COLOR_GREY_500),
                )]),
                Line::from(vec![Span::styled(
                    author.email().unwrap_or("").to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(author.when()),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    format!("committed by: {}", committer.name().unwrap_or("-")),
                    Style::default().fg(COLOR_GREY_500),
                )]),
                Line::from(vec![Span::styled(
                    committer.email().unwrap_or("").to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(committer.when()).to_string(),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "message summary:",
                    Style::default().fg(COLOR_GREY_500),
                )])
            ]);

            let wrapped = clean_commit_text(&summary, max_text_width);
            for wrap in wrapped {
                commit_lines.push(Line::from(vec![Span::styled(
                    wrap,
                    Style::default().fg(COLOR_TEXT),
                )]));
            }
            
            commit_lines.extend(vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "message body:",
                    Style::default().fg(COLOR_GREY_500),
                )])
            ]);

            let wrapped = clean_commit_text(&body, max_text_width);
            for wrap in wrapped {
                commit_lines.push(Line::from(vec![Span::styled(
                    wrap,
                    Style::default().fg(COLOR_TEXT),
                )]));
            }

        }

        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };

        let total_lines = commit_lines.len();
        let visible_height = self.layout.inspector.height as usize - 2;

        if total_lines == 0 {
            self.inspector_selected = 0;
        } else if self.inspector_selected >= total_lines {
            self.inspector_selected = total_lines - 1;
        }

        self.ensure_inspector_selected_visible(total_lines, visible_height);

        let scroll_offset = self.inspector_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (scroll_offset + visible_height).min(total_lines);

        let items: Vec<ListItem> = commit_lines[scroll_offset..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let absolute_idx = scroll_offset + i;
                let mut item = ListItem::new(line.clone());
                if absolute_idx == self.inspector_selected {
                    item = item.style(Style::default().bg(COLOR_GREY_800));
                }
                item
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .padding(padding)
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(
                            " (i)nspector ",
                            Style::default().fg(if self.focus == Focus::Inspector {
                                COLOR_GREY_500
                            } else {
                                COLOR_TEXT
                            }),
                        ),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_500))
                    .borders(if self.is_status {
                        Borders::RIGHT | Borders::TOP
                    } else {
                        Borders::RIGHT | Borders::TOP | Borders::BOTTOM
                    })
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .bg(COLOR_GREY_800)
                    .fg(COLOR_TEXT),
            )
            .repeat_highlight_symbol(false);

        frame.render_widget(list, self.layout.inspector);

        let mut scrollbar_state =
            ScrollbarState::new(total_lines).position(self.inspector_scroll.get());

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(if self.is_status { Some("│") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol(if total_lines > visible_height { "▌" } else { "│" })
            .thumb_style(Style::default().fg(if total_lines > visible_height {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        frame.render_stateful_widget(scrollbar, self.layout.inspector, &mut scrollbar_state);
    }
}