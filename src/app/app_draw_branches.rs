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
    helpers::{
        palette::*,
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

    pub fn draw_branches(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.branches.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        // Lines
        let mut lines: Vec<Line<'_>> = Vec::new();
        for (oid, branch) in self.oid_branch_vec.iter() {
            let is_visible = self.visible_branch_oids.contains(oid);
            lines.push(Line::from(vec![
                Span::styled(format!("{} {}", if is_visible {"●"} else {"◌"}, branch), Style::default().fg(
                    if is_visible {*self.tip_colors.get(&oid).unwrap_or(&COLOR_GRASS)}
                    else {COLOR_TEXT}
                ))
            ]));
        }
        // Line::from(vec![Span::styled("commit sha:", Style::default().fg(COLOR_GREY_500))]);

        // Get vertical dimensions
        let total_lines = lines.len();
        let visible_height = self.layout.branches.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.branches_selected = 0;
        } else if self.branches_selected >= total_lines {
            self.branches_selected = total_lines - 1;
        }
        
        // Trap selection
        self.trap_selection(self.branches_selected, &self.branches_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.branches_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if start + i == self.branches_selected && self.focus == Focus::Branches {
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
                    //     Span::styled(
                    //         " (i)nspector ",
                    //         Style::default().fg(if self.focus == Focus::Branches {
                    //             COLOR_GREY_500
                    //         } else {
                    //             COLOR_TEXT
                    //         }),
                    //     ),
                    //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    // ])
                    // .title_alignment(Alignment::Right)
                    // .title_style(Style::default().fg(COLOR_GREY_500))
            );

        frame.render_widget(list, self.layout.branches);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.branches_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("─"))
            .end_symbol(Some("─"))
            .track_symbol(Some("│"))
            .thumb_symbol(if total_lines > visible_height { "▌" } else { "│" })
            .thumb_style(Style::default().fg(if total_lines > visible_height && self.focus == Focus::Branches {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.branches_scrollbar, &mut scrollbar_state);
    }
}