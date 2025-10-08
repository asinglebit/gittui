#[rustfmt::skip]
use std::{
    cell::Cell,
    collections::HashMap,
    env,
    io,
    path::PathBuf
};
#[rustfmt::skip]
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use ratatui::{
    DefaultTerminal,
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
    core::walker::walk,
    git::{
        actions::checkout,
        queries::{
            get_changed_filenames_as_text,
            get_current_branch
        },
    },
    utils::{
        colors::*,
        time::timestamp_to_utc
    },
};

pub struct App {
    // General
    path: String,
    repo: Repository,

    // Data
    oids: Vec<Oid>,
    tips: HashMap<Oid, Vec<String>>,

    // Lines
    lines_graph: Vec<Line<'static>>,
    lines_branches: Vec<Line<'static>>,
    lines_messages: Vec<Line<'static>>,
    lines_buffers: Vec<Line<'static>>,

    // Interface
    scroll: Cell<usize>,
    files_scroll: Cell<usize>,
    selected: usize,
    modal: bool,
    minimal: bool,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.reload();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn reload(&mut self) {
        let walked = walk(&self.repo);
        self.oids = walked.oids;
        self.tips = walked.tips;
        self.lines_graph = walked.lines_graph;
        self.lines_branches = walked.lines_branches;
        self.lines_messages = walked.lines_messages;
        self.lines_buffers = walked.lines_buffer;
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        /***************************************************************************************************
         * Layout
         ***************************************************************************************************/

        let chunks_vertical = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(if self.minimal { 0 } else { 1 }),
                ratatui::layout::Constraint::Percentage(100),
                ratatui::layout::Constraint::Length(if self.minimal { 0 } else { 1 }),
            ])
            .split(frame.area());

        let chunks_title_bar = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(80),
                ratatui::layout::Constraint::Percentage(20),
            ])
            .split(chunks_vertical[0]);

        let chunks_horizontal = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(70),
                ratatui::layout::Constraint::Percentage(30),
            ])
            .split(chunks_vertical[1]);

        let chunks_inspector = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Percentage(40),
                ratatui::layout::Constraint::Percentage(60),
            ])
            .split(chunks_horizontal[1]);

        let chunks_status_bar = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(80),
                ratatui::layout::Constraint::Percentage(20),
            ])
            .split(chunks_vertical[2]);

        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };

        /***************************************************************************************************
         * Title bar
         ***************************************************************************************************/
        let current_branch_name = match get_current_branch(&self.repo) {
            Some(branch) => format!(" ● {}", branch),
            None => format!(" ○ HEAD: {}", self.repo.head().unwrap().target().unwrap()),
        };

        let sha_paragraph = ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![
            Span::styled(" GUITAR |", Style::default().fg(COLOR_TEXT)),
            Span::styled(current_branch_name, Style::default().fg(COLOR_TEXT)),
        ])))
        .left_aligned()
        .block(Block::default());

        frame.render_widget(sha_paragraph, chunks_title_bar[0]);

        /***************************************************************************************************
         * Status bar
         ***************************************************************************************************/

        let status_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(Line::from(vec![Span::styled(
                format!(" 🖿  {}", self.path),
                Style::default().fg(COLOR_TEXT),
            )])))
            .left_aligned()
            .block(Block::default());

        frame.render_widget(status_paragraph, chunks_status_bar[0]);

        let title_paragraph =
            ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(
                format!("{}/{}", self.selected + 1, self.lines_messages.len()),
                Style::default().fg(COLOR_TEXT),
            ))))
            .right_aligned()
            .block(Block::default());

        frame.render_widget(Clear, chunks_status_bar[1]);
        frame.render_widget(title_paragraph, chunks_status_bar[1]);

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

        let visible_height = chunks_inspector[0].height as usize;
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
                let wrapped_lines = (visual_width + chunks_inspector[0].width as usize)
                    / chunks_inspector[0].width as usize;
                wrapped_lines.max(1) // at least 1 line
            })
            .sum::<usize>();

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

        frame.render_widget(commit_paragraph, chunks_inspector[0]);

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

        frame.render_stateful_widget(scrollbar, chunks_inspector[0], &mut scrollbar_state);

        /***************************************************************************************************
         * Files
         ***************************************************************************************************/

        let mut files_text: Text = Text::from("-");
        let sha: Oid = *self.oids.get(self.selected).unwrap();
        if sha != Oid::zero() {
            files_text = get_changed_filenames_as_text(&self.repo, sha);
        }
        let total_file_lines = files_text.lines.len();
        let visible_height = chunks_inspector[1].height as usize;
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

        frame.render_widget(files_paragraph, chunks_inspector[1]);

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

        frame.render_stateful_widget(scrollbar, chunks_inspector[1], &mut scrollbar_state);

        /***************************************************************************************************
         * Graph table
         ***************************************************************************************************/

        let table_height = chunks_horizontal[0].height as usize - 2;
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

        frame.render_widget(Clear, chunks_horizontal[0]);

        frame.render_widget(table, chunks_horizontal[0]);

        // Render the scrollbar
        let total_lines = self.oids.len();
        let visible_height = chunks_inspector[0].height as usize;
        if total_lines > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_lines).position(self.scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("─"))
                .end_symbol(Some("─"))
                .track_symbol(Some("│"))
                .thumb_symbol("▌")
                .thumb_style(Style::default().fg(COLOR_GREY_600));

            frame.render_stateful_widget(scrollbar, chunks_horizontal[0], &mut scrollbar_state);
        }

        /***************************************************************************************************
         * Modal
         ***************************************************************************************************/

        if self.modal {
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

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.lines_branches.len() && !self.modal {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 && !self.modal {
                    self.selected -= 1;
                }
            }
            KeyCode::Char('f') => {
                self.minimal = !self.minimal;
            }
            KeyCode::Home => {
                if !self.modal {
                    self.selected = 0;
                }
            }
            KeyCode::End => {
                if !self.lines_branches.is_empty() && !self.modal {
                    self.selected = self.lines_branches.len() - 1;
                }
            }
            KeyCode::PageUp => {
                if !self.modal {
                    let page = 20;
                    if self.selected >= page {
                        self.selected -= page;
                    } else {
                        self.selected = 0;
                    }
                }
            }
            KeyCode::PageDown => {
                if !self.modal {
                    let page = 20;
                    if self.selected + page < self.lines_branches.len() {
                        self.selected += page;
                    } else {
                        self.selected = self.lines_branches.len() - 1;
                    }
                }
            }
            KeyCode::Enter => {
                if !self.modal {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.selected).unwrap())
                        .or_default();
                    if branches.len() > 1 {
                        self.modal = true;
                    } else {
                        checkout(&self.repo, *self.oids.get(self.selected).unwrap());
                        self.reload();
                    }
                }
            }
            KeyCode::Esc => {
                if self.modal {
                    self.modal = false;
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let path = if args.len() > 1 {
            &args[1]
        } else {
            &".".to_string()
        };
        let absolute_path: PathBuf = std::fs::canonicalize(path)
            .unwrap_or_else(|_| PathBuf::from(path));
        let repo = Repository::open(absolute_path.clone()).expect("Could not open repo");

        App {
            // General
            path: absolute_path.display().to_string(),
            repo,

            // Data
            oids: Vec::new(),
            tips: HashMap::new(),
            lines_graph: Vec::new(),
            lines_branches: Vec::new(),
            lines_messages: Vec::new(),
            lines_buffers: Vec::new(),

            // Interface
            scroll: 0.into(),
            files_scroll: 0.into(),
            selected: 0,
            modal: false,
            minimal: false,
            exit: false,
        }
    }
}
