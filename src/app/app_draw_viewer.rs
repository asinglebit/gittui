#[rustfmt::skip]
use git2::Oid;
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
    }
};
#[rustfmt::skip]
use crate::{
    helpers::{
        palette::*
    },
};
#[rustfmt::skip]
use crate::{
    app::app::{
        App,
        Focus,
        Viewport
    },
    git::{
        queries::{
            diffs::{
                get_file_at_oid,
                get_file_at_workdir,
                get_file_diff_at_oid,
                get_file_diff_at_workdir
            }
        }
    },
    helpers::{
        text::{
            wrap_words
        }
    }
};

impl App {

    pub fn draw_viewer(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.graph.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        // Get vertical dimensions
        let total_lines = self.viewer_lines.len();
        let visible_height = self.layout.graph.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.viewer_selected = 0;
        } else if self.viewer_selected >= total_lines {
            self.viewer_selected = total_lines - 1;
        }
        
        // Trap selection
        self.trap_selection(self.viewer_selected, &self.viewer_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.viewer_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = self.viewer_lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let absolute_idx = start + i;
                let mut item = line.clone();
                if absolute_idx == self.viewer_selected && self.focus == Focus::Viewport {
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
                        Span::styled(" viewer ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        // Render the list
        frame.render_widget(list, self.layout.graph);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.viewer_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╮") })
            .end_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol("▌")
            .thumb_style(Style::default().fg(if self.focus == Focus::Viewport {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.graph, &mut scrollbar_state);
    }

    pub fn open_viewer(&mut self) {
        match self.focus {
            Focus::StatusTop => {
                if self.graph_selected != 0 && self.current_diff.len() > 0 {
                    self.file_name = Some(
                        self.current_diff
                            .get(self.status_top_selected)
                            .unwrap()
                            .filename
                            .to_string(),
                    );
                    self.update_viewer(self.oids.get(self.graph_selected).unwrap().clone());
                    self.viewport = Viewport::Viewer;
                } else if self.graph_selected == 0 && self.uncommitted.is_staged {
                    let modified_len = self.uncommitted.staged.modified.len();
                    let added_len = self.uncommitted.staged.added.len();
                    let index = self.status_top_selected;
                    self.file_name = if index < modified_len {
                        self.uncommitted.staged.modified.get(index).cloned()
                    } else if index < modified_len + added_len {
                        self.uncommitted
                            .staged
                            .added
                            .get(index - modified_len)
                            .cloned()
                    } else {
                        self.uncommitted
                            .staged
                            .deleted
                            .get(index - modified_len - added_len)
                            .cloned()
                    };
                    self.update_viewer(Oid::zero());
                    self.viewport = Viewport::Viewer;
                }
            }
            Focus::StatusBottom => {
                if self.graph_selected == 0 && self.uncommitted.is_unstaged {
                    let modified_len = self.uncommitted.unstaged.modified.len();
                    let added_len = self.uncommitted.unstaged.added.len();
                    let index = self.status_bottom_selected;
                    self.file_name = if index < modified_len {
                        self.uncommitted.unstaged.modified.get(index).cloned()
                    } else if index < modified_len + added_len {
                        self.uncommitted
                            .unstaged
                            .added
                            .get(index - modified_len)
                            .cloned()
                    } else {
                        self.uncommitted
                            .unstaged
                            .deleted
                            .get(index - modified_len - added_len)
                            .cloned()
                    };
                    self.update_viewer(Oid::zero());
                    self.viewport = Viewport::Viewer;
                }
            }
            _ => {}
        }
    }
    
    pub fn update_viewer(&mut self, oid: Oid) {
        
        let filename = self.file_name.clone().unwrap();

        // Decide whether to use committed or uncommitted version
        let (original_lines, hunks) = if oid == Oid::zero() {(
            get_file_at_workdir(&self.repo, &filename),
            get_file_diff_at_workdir(&self.repo, &filename).unwrap_or_default(),
        )} else {(
            get_file_at_oid(&self.repo, oid, &filename),
            get_file_diff_at_oid(&self.repo, oid, &filename).unwrap_or_default(),
        )};

        self.viewer_lines.clear();
        let mut current_line: usize = 0;
        let mut current_line_old: usize = 0;

        for hunk in hunks.iter() {
            // Parse hunk header to extract start line and length for the old file.
            // Example header: "@@ -22,8 +22,14 @@"
            let header = &hunk.header;
            let (old_start, _old_len) = header
                .split_whitespace()
                .nth(1) // "-22,8"
                .and_then(|s| s.strip_prefix('-'))
                .and_then(|s| {
                    let mut parts = s.split(',');
                    Some((
                        parts.next()?.parse::<usize>().ok()?,
                        parts
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0),
                    ))
                })
                .unwrap_or((1, 0));
            let old_start_idx = old_start.saturating_sub(1);

            // Add unchanged lines before this hunk
            while current_line < old_start_idx && current_line < original_lines.len() {
                let wrapped = wrap_words(
                    original_lines[current_line].clone(),
                    (self.layout.graph.width as usize).saturating_sub(8),
                );
                let mut idx = 0;
                for line in wrapped {
                    self.viewer_lines.push(ListItem::new(
                        Line::from(vec![
                            Span::styled(
                                format!("{}", if idx == 0 { format!("{:3}  ", current_line + 1) } else { format!("     ") }),
                                Style::default().fg(COLOR_BORDER),
                            ),
                            Span::styled(format!("{}", line), Style::default().fg(COLOR_GREY_500)),
                        ])
                        .style(Style::default()),
                    ));
                    idx += 1;
                }
                current_line += 1;
                current_line_old += 1;
            }
            
            // Process lines in the hunk
            for line in hunk.lines.iter().filter(|l| l.origin != 'H') {
                let text = line.content.trim_end_matches('\n');

                let (style, prefix, side, fg, count) = match line.origin {
                    '-' => (Style::default().bg(COLOR_DARK_RED).fg(COLOR_RED), "- ".to_string(), COLOR_RED, COLOR_RED, current_line_old + 1),
                    '+' => (Style::default().bg(COLOR_LIGHT_GREEN_900).fg(COLOR_GREEN), "+ ".to_string(), COLOR_GREEN, COLOR_GREEN, current_line + 1),
                    ' ' => (Style::default(), "".to_string(), COLOR_BORDER, COLOR_GREY_500, current_line + 1),
                    _ => (Style::default(), "".to_string(), COLOR_BORDER, COLOR_GREY_500, 0)
                };

                let wrapped = wrap_words(format!("{}{}", prefix, text), (self.layout.graph.width as usize).saturating_sub(9));
                let mut idx = 0;

                for line in wrapped {
                    self.viewer_lines.push(
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{}",if idx == 0 {format!("{:3}  ", count)} else {format!("     ")}),
                                Style::default().fg(side),
                            ),
                            Span::styled(format!("{}", line), Style::default().fg(fg))
                        ]))
                        .style(style),
                    );
                    idx += 1;
                }

                match line.origin {
                    '-' => {
                        current_line_old += 1;
                    }
                    '+' => {
                        current_line += 1;
                    }
                    ' ' => {
                        current_line += 1;
                        current_line_old += 1;
                    }
                    _ => {}
                }
            }
        }

        // Add remaining lines after the last hunk
        while current_line < original_lines.len() {
            let wrapped = wrap_words(
                original_lines[current_line].clone(),
                (self.layout.graph.width as usize).saturating_sub(8),
            );
            let mut idx = 0;
            for line in wrapped {
                self.viewer_lines.push(
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{}", if idx == 0 {format!("{:3}  ", current_line + 1)} else {format!("     ")}),
                            Style::default().fg(COLOR_BORDER),
                        ),
                        Span::styled(format!("{}", line), Style::default().fg(COLOR_GREY_500)),
                    ]))
                    .style(Style::default()),
                );
                idx += 1;
            }
            current_line += 1;
        }
    }
}
