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
            let oid: Oid = *self.oids.get(self.graph_selected).unwrap();
            let commit = self.repo.find_commit(oid).unwrap();
            let author = commit.author();
            let committer = commit.committer();
            let summary = commit.summary().unwrap_or("<no summary>").to_string();
            let body = commit.body().unwrap_or("<no body>").to_string();
            let branches = self.oid_branch_map.get(&oid).unwrap();

            // Assemble lines
            lines = vec![
                Line::from(vec![Span::styled("commit sha:", Style::default().fg(COLOR_GREY_500))]),
                Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("#{}", oid), max_text_width),
                    Style::default().fg(*self.oid_colors.get(&oid).unwrap()),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled("parent shas:", Style::default().fg(COLOR_GREY_500))]),
            ];

            for parent_id in commit.parent_ids() {
                lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("#{}", parent_id), max_text_width),
                    Style::default().fg(*self.oid_colors.get(&parent_id).unwrap()),
                )]));
            }

            lines.extend(vec![
                Line::from(""),
            ]);

            lines.push(Line::from(vec![Span::styled(
                "featured branches:",
                Style::default().fg(COLOR_GREY_500),
            )]));

            for branch in branches {
                let oid = self.branch_oid_map.get(branch).unwrap();
                let color = self.tip_colors.get(oid).unwrap();
                lines.push(Line::from(vec![Span::styled(
                    truncate_with_ellipsis(&format!("● {}", branch), max_text_width),
                    Style::default().fg(*color),
                )]));
            }

            lines.extend(vec![
                Line::from(""),
            ]);

            lines.extend(vec![
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
                lines.push(Line::from(vec![Span::styled(
                    wrap,
                    Style::default().fg(COLOR_TEXT),
                )]));
            }
            
            lines.extend(vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    "message body:",
                    Style::default().fg(COLOR_GREY_500),
                )])
            ]);

            let wrapped = clean_commit_text(&body, max_text_width);
            for wrap in wrapped {
                lines.push(Line::from(vec![Span::styled(
                    wrap,
                    Style::default().fg(COLOR_TEXT),
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
        let scroll_offset = self.inspector_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (scroll_offset + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = lines[scroll_offset..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let absolute_idx = scroll_offset + i;
                let mut item = ListItem::new(line.clone());
                if absolute_idx == self.inspector_selected && self.focus == Focus::Inspector {
                    item = item.style(Style::default().bg(COLOR_GREY_800));
                }
                item
            })
            .collect();
        
        // Setup the list
        let list = List::new(list_items)
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
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.inspector, &mut scrollbar_state);
    }
}