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
    widgets::{
        Block,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        List,
        ListItem
    },
};
#[rustfmt::skip]
use crate::{
    helpers::{
        text::{
            truncate_with_ellipsis,
            sanitize,
            wrap_words
        },
        time::timestamp_to_utc
    },
    app::app::{
        App,
        Focus
    },
};

impl App {

    pub fn draw_inspector(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.inspector.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        // Flags
        let is_showing_uncommitted = self.graph_selected == 0;

        // Lines
        let mut lines: Vec<Line<'_>> = Vec::new();

        if !is_showing_uncommitted {
            
            // Query commit info
            let zero = Oid::zero();
            let oidi = self.commit_manager.oidi_sorted.get(self.graph_selected).unwrap();
            let oid = self.commit_manager.oidi_to_oid.get(*oidi as usize).unwrap_or(&zero);
            
            
            let commit = self.repo.find_commit(*oid).unwrap();
            let author = commit.author();
            let committer = commit.committer();
            let summary = commit.summary().unwrap_or("⊘ no summary").to_string();
            let body = commit.body().unwrap_or("⊘ no body").to_string();

            // Assemble lines
            lines = vec![
                Line::from(vec![Span::styled("commit sha:", Style::default().fg(self.theme.COLOR_GREY_500))]),
                Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("#{}", oid), max_text_width),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]),
                Line::default(),
                Line::from(vec![Span::styled("parent shas:", Style::default().fg(self.theme.COLOR_GREY_500))]),
            ];

            for parent_id in commit.parent_ids() {
                lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("#{}", parent_id), max_text_width),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]));
            }

            if let Some(branches) = self.branch_manager.tips.get(oidi)
                && let Some(color) = self.branch_manager.tip_colors.get(oidi) {
                    lines.extend(vec![
                        Line::default(),
                    ]);
                    lines.push(Line::from(vec![Span::styled(
                        "featured branches:",
                        Style::default().fg(self.theme.COLOR_GREY_500),
                    )]));
                    for branch in branches {
                        lines.push(Line::from(vec![
                            Span::styled(
                                truncate_with_ellipsis(&format!("● {}", branch), max_text_width),
                                Style::default().fg(*color),
                            )
                        ]));
                    }
                }

            lines.extend(vec![
                Line::default(),
            ]);

            lines.extend(vec![
                Line::from(vec![Span::styled(
                    format!("authored by: {}", author.name().unwrap_or("-")),
                    Style::default().fg(self.theme.COLOR_GREY_500),
                )]),
                Line::from(vec![Span::styled(
                    author.email().unwrap_or("").to_string(),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(author.when()),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]),
                Line::default(),
                Line::from(vec![Span::styled(
                    format!("committed by: {}", committer.name().unwrap_or("-")),
                    Style::default().fg(self.theme.COLOR_GREY_500),
                )]),
                Line::from(vec![Span::styled(
                    committer.email().unwrap_or("").to_string(),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    timestamp_to_utc(committer.when()).to_string(),
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]),
                Line::default(),
                Line::from(vec![Span::styled(
                    "message summary:",
                    Style::default().fg(self.theme.COLOR_GREY_500),
                )])
            ]);

            let wrapped = wrap_words(sanitize(summary), max_text_width);
            for line in wrapped {
                lines.push(Line::from(vec![Span::styled(
                    line,
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]));
            }
            
            lines.extend(vec![
                Line::default(),
                Line::from(vec![Span::styled(
                    "message body:",
                    Style::default().fg(self.theme.COLOR_GREY_500),
                )])
            ]);

            let wrapped = wrap_words(sanitize(body), max_text_width);
            for line in wrapped {
                lines.push(Line::from(vec![Span::styled(
                    line,
                    Style::default().fg(self.theme.COLOR_TEXT),
                )]));
            }
        }

        // Get vertical dimensions
        let total_lines = lines.len();
        let visible_height = self.layout.inspector.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.inspector_selected = 0;
        } else if self.inspector_selected >= total_lines {
            self.inspector_selected = total_lines - 1;
        }
        
        // Trap selection
        self.trap_selection(self.inspector_selected, &self.inspector_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.inspector_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if start + i == self.inspector_selected && self.focus == Focus::Inspector {
                    let spans: Vec<Span> = line.iter().map(|span| { Span::styled(span.content.clone(), span.style.fg(self.theme.COLOR_GREY_500)) }).collect();
                    ListItem::new(Line::from(spans)).style(Style::default().bg(self.theme.COLOR_GREY_800).fg(self.theme.COLOR_GREY_500))
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
            );

        frame.render_widget(list, self.layout.inspector);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.inspector_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(if self.is_status { Some("│") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol(if total_lines > visible_height { "▌" } else { "│" })
            .thumb_style(Style::default().fg(if total_lines > visible_height && self.focus == Focus::Inspector {
                self.theme.COLOR_GREY_600
            } else {
                self.theme.COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.inspector_scrollbar, &mut scrollbar_state);
    }
}