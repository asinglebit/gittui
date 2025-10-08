#[rustfmt::skip]
#[rustfmt::skip]
use git2::{
    Oid
};
#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    layout::{
        Alignment,
        Rect
    },
    text::{
        Line,
        Span,
        Text
    },
    widgets::{
        Block,
        Borders,
        Cell as WidgetCell,
        Clear,
        Paragraph,
        Row,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        Table,
        Widget,
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
    app::{
        layout::{
            layout::generate_layout,
            title::render_title_bar,
            status::render_status_bar
        }
    },
    utils::{
        colors::*,
        time::timestamp_to_utc
    },
};
#[rustfmt::skip]
use crate::app::app::App;

impl App {

    pub fn draw(&mut self, frame: &mut Frame) {

        let layout = generate_layout(&frame, self.is_minimal);
        render_title_bar(frame, &self.repo, &layout);
        render_status_bar(frame, &layout, self.selected, &self.lines_messages, &self.path);

        /***************************************************************************************************
         * Inspector
         ***************************************************************************************************/

        let mut commit_lines: Vec<Line<'_>> = Vec::new();
        let sha: Oid = *self.oids.get(self.selected).unwrap();
        if sha != Oid::zero() {
            let commit = self.repo.find_commit(sha).unwrap();
            let author = commit.author();
            let committer = commit.committer();
            let summary = commit.summary().unwrap_or("<no summary>").to_string();
            let body = commit.body().unwrap_or("<no body>").to_string();

            commit_lines = vec![
                Line::from(vec![Span::styled(
                    "Commit sha:",
                    Style::default().fg(COLOR_GREY_400),
                )]),
                Line::from(vec![Span::styled(
                    format!("{}", self.oids.get(self.selected).unwrap()),
                    Style::default().fg(COLOR_TEXT),
                )]),
                Line::from(vec![Span::styled(
                    "Parent shas:",
                    Style::default().fg(COLOR_GREY_400),
                )]),
            ];

            for parent_id in commit.parent_ids() {
                commit_lines.push(Line::from(vec![Span::styled(
                    format!("{}", parent_id),
                    Style::default().fg(COLOR_TEXT),
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

        let visible_height = layout.inspector.height as usize;
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
                let wrapped_lines = (visual_width + layout.inspector.width as usize)
                    / layout.inspector.width as usize;
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
            // .wrap(Wrap { trim: true }) For some reasone causes ghosting
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" Inspector ", Style::default().fg(COLOR_TEXT)),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::RIGHT | Borders::TOP)
                    .border_style(Style::default().fg(COLOR_BORDER))
                    .padding(padding)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        frame.render_widget(commit_paragraph, layout.inspector);

        // Render the scrollbar
        let mut scrollbar_state =
            ScrollbarState::new(total_inspector_lines).position(self.files_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(Some("│"))
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

        frame.render_stateful_widget(scrollbar, layout.inspector, &mut scrollbar_state);

        /***************************************************************************************************
         * Files
         ***************************************************************************************************/

        
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
        let visible_height = layout.files.height as usize;
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

        frame.render_widget(files_paragraph, layout.files);

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

        frame.render_stateful_widget(scrollbar, layout.files, &mut scrollbar_state);

        /***************************************************************************************************
         * Graph table
         ***************************************************************************************************/

        let table_height = layout.graph.height as usize - 2;
        let total_rows = self.lines_graph.len();

        // Make sure selected row is visible
        if self.selected < self.scroll.get() {
            self.scroll.set(self.selected);
        } else if self.selected >= self.scroll.get() + table_height {
            self.scroll
                .set(self.selected.saturating_sub(table_height - 1));
        }

        let start = self.scroll.get();
        let end = (self.scroll.get() + table_height).min(total_rows);

        // Start with fake commit row
        let mut rows = Vec::with_capacity(end - start + 1); // preallocate for efficiency

        // Add the rest of the commits
        for (i, ((graph, branch), buffer)) in self.lines_graph[start..end]
            .iter()
            .zip(&self.lines_branches[start..end])
            .zip(&self.lines_buffers[start..end])
            .enumerate()
        {
            let actual_index = start + i;
            let mut row = Row::new(vec![
                WidgetCell::from(graph.clone()),
                WidgetCell::from(branch.clone()),
                WidgetCell::from(buffer.clone()),
            ]);

            if actual_index == self.selected {
                row = row.style(Style::default().bg(COLOR_GREY_800).fg(COLOR_GREY_600));
            }
            rows.push(row);
        }

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(25),
                ratatui::layout::Constraint::Percentage(100),
            ],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .row_highlight_style(Style::default().bg(COLOR_SELECTION).fg(COLOR_TEXT_SELECTED))
        .column_spacing(2);

        frame.render_widget(Clear, layout.graph);

        frame.render_widget(table, layout.graph);

        // Render the scrollbar
        let total_lines = self.oids.len();
        let visible_height = layout.graph.height as usize;
        if total_lines > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_lines).position(self.scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("─"))
                .end_symbol(Some("─"))
                .track_symbol(Some("│"))
                .thumb_symbol("▌")
                .thumb_style(Style::default().fg(COLOR_GREY_600));

            frame.render_stateful_widget(scrollbar, layout.graph, &mut scrollbar_state);
        }

        /***************************************************************************************************
         * Modal
         ***************************************************************************************************/

        if self.is_modal {
            let mut length = 0;
            let branches = self
                .tips
                .entry(*self.oids.get(self.selected).unwrap())
                .or_default();
            let spans: Vec<Line> = branches
                .iter()
                .map(|branch_name| {
                    length = (10 + branch_name.len()).max(length);
                    Line::from(Span::styled(
                        format!("● {} ", branch_name),
                        Style::default().fg(COLOR_GREY_400),
                    ))
                })
                .collect();
            let height = branches.len() + 4;

            let bg_block = Block::default().style(Style::default().fg(COLOR_BORDER));
            bg_block.render(frame.area(), frame.buffer_mut());

            // Modal size (smaller than area)
            let modal_width = length.min((frame.area().width as f32 * 0.8) as usize) as u16;
            let modal_height = height.min((frame.area().height as f32 * 0.6) as usize) as u16;
            let x = frame.area().x + (frame.area().width - modal_width) / 2;
            let y = frame.area().y + (frame.area().height - modal_height) / 2;
            let modal_area = Rect::new(x, y, modal_width, modal_height);

            frame.render_widget(Clear, modal_area);

            let padding = ratatui::widgets::Padding {
                left: 3,
                right: 3,
                top: 1,
                bottom: 1,
            };

            // Modal block
            let modal_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_GREY_600))
                .title(Span::styled(" x ", Style::default().fg(COLOR_GREY_500)))
                .title_alignment(Alignment::Right)
                .padding(padding)
                .border_type(ratatui::widgets::BorderType::Rounded);

            // Modal content

            let paragraph = Paragraph::new(Text::from(spans))
                .block(modal_block)
                .alignment(Alignment::Center);

            paragraph.render(modal_area, frame.buffer_mut());
        }
    }
}