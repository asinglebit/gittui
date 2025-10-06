use std::{cell::Cell, env, io, path::PathBuf};

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use git2::{Oid, Repository, Time};
use ratatui::{
    buffer::Buffer, layout::{Rect}, text::{Line, Span, Text}, widgets::{Block, Borders, Cell as WidgetCell, Padding, Row, Table, Widget, Wrap}, DefaultTerminal, Frame
};
use ratatui::style::{Style};

use crate::helpers::get_commits;
use crate::colors::*;

pub struct App {
    // General
    path: String,
    repo: Repository,
    
    // Data
    shas: Vec<Oid>,
    graph: Vec<Line<'static>>,
    branches: Vec<Line<'static>>,
    messages: Vec<Line<'static>>,
    buffers: Vec<Line<'static>>,
    
    // Interface
    scroll: Cell<usize>,
    selected: usize,
    exit: bool
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
        let (shas, graph, branches, messages, buffer) = get_commits(&self.repo);
        self.shas = shas;
        self.graph = graph;
        self.branches = branches;
        self.messages = messages;
        self.buffers = buffer;
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => self.exit(),         
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected + 1 < self.branches.len() { self.selected += 1; }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 { self.selected -= 1; }
            }
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                if !self.branches.is_empty() {
                    self.selected = self.branches.len() - 1;
                }
            }
            KeyCode::PageUp => {
                let page = 20;
                if self.selected >= page {
                    self.selected -= page;
                } else {
                    self.selected = 0;
                }
            }
            KeyCode::PageDown => {
                let page = 20;
                if self.selected + page < self.branches.len() {
                    self.selected += page;
                } else {
                    self.selected = self.branches.len() - 1;
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
        let path = if args.len() > 1 { &args[1] } else { &".".to_string() };
        let absolute_path: PathBuf = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(path));
        // let path_buf = PathBuf::from(&path);
        let repo = Repository::open(absolute_path.clone()).expect("Could not open repo");
        
        App {
            // General
            path: absolute_path.display().to_string(),
            repo,
            
            // Data
            shas: Vec::new(),
            graph: Vec::new(),
            branches: Vec::new(),
            messages: Vec::new(),
            buffers: Vec::new(),
            
            // Interface
            scroll: 0.into(),
            selected: 0,
            exit: false
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        // Layout
        let chunks_vertical = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([ratatui::layout::Constraint::Length(1), ratatui::layout::Constraint::Percentage(100), ratatui::layout::Constraint::Length(1)])
            .split(area);
        let chunks_title_bar = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(80), ratatui::layout::Constraint::Percentage(20)])
            .split(chunks_vertical[0]);
        let chunks_horizontal = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(70), ratatui::layout::Constraint::Percentage(30)])
            .split(chunks_vertical[1]);
        let chunks_inspector = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([ratatui::layout::Constraint::Percentage(40), ratatui::layout::Constraint::Percentage(60)])
            .split(chunks_horizontal[1]);
        let chunks_status_bar = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(80), ratatui::layout::Constraint::Percentage(20)])
            .split(chunks_vertical[2]);

        // Title
        ratatui::widgets::Paragraph::new(Text::from(
            Line::from(vec![
                Span::styled(format!(" GitTui |"), Style::default().fg(COLOR_TITLE)),
                Span::styled(format!(" 🖿 {}", self.path), Style::default().fg(COLOR_TEXT))
            ])
        ))
            .left_aligned()
            .block(ratatui::widgets::Block::default())
            .render(chunks_title_bar[0], buf);

        // Status bar
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(format!(" SHA: {}", self.shas.get(self.selected).unwrap()), Style::default().fg(COLOR_TEXT)))))
            .left_aligned()
            .block(ratatui::widgets::Block::default())
            .render(chunks_status_bar[0], buf);
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(format!("{}/{} ", self.selected + 1, self.messages.len()), Style::default().fg(COLOR_TITLE)))))
            .right_aligned()
            .block(ratatui::widgets::Block::default())
            .render(chunks_status_bar[1], buf);

        let table_height = chunks_horizontal[0].height as usize - 3; // visible rows
        let total_rows = self.graph.len(); // assume all columns have same length

        // Make sure selected row is visible
        if self.selected < self.scroll.get() {
            self.scroll.set(self.selected);
        } else if self.selected >= self.scroll.get() + table_height {
            self.scroll.set(self.selected.saturating_sub(table_height - 1));
        }

        // Compute visible slice
        let start = self.scroll.get();
        let end = (self.scroll.get() + table_height).min(total_rows);
        
        // Build table rows
        let rows: Vec<Row> = self.graph[start..end]
            .iter()
            .zip(&self.branches[start..end])
            .zip(&self.buffers[start..end])
            .enumerate()
            .map(|(i, ((graph, branch), buffer))| {
                let actual_index = start + i; // absolute row index
                let mut row = Row::new(vec![
                    WidgetCell::from(graph.clone()),
                    WidgetCell::from(branch.clone()),
                    WidgetCell::from(buffer.clone()),
                ]);

                if actual_index == self.selected {
                    row = row.style(
                        Style::default()
                            .bg(COLOR_GREY_800)
                            .fg(COLOR_GREY_400),
                    );
                }

                row
            })
            .collect();

        // Build table with headers
        // let header = Row::new(vec![
        //     WidgetCell::from("History").style(Style::default().fg(COLOR_TITLE)),
        //     WidgetCell::from("Messages").style(Style::default().fg(COLOR_TITLE)),
        // ]);

        let table = Table::new(
            rows,
            [
                ratatui::layout::Constraint::Length(25),
                ratatui::layout::Constraint::Percentage(100),
            ],
        )
        // .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .border_type(ratatui::widgets::BorderType::Rounded),
        )
        .row_highlight_style(
            Style::default()
                .bg(COLOR_SELECTION)
                .fg(COLOR_TEXT_SELECTED),
        )
        .column_spacing(2);

        // Render table in middle chunk
        buf.set_style(chunks_horizontal[0], Style::default()); // clear area
        table.render(chunks_horizontal[0], buf);

        // Commit info
        let commit = self.repo.find_commit(*self.shas.get(self.selected).unwrap()).unwrap();
        let author = commit.author();
        let committer = commit.committer();
        let message = commit.message();

        // Build parent lines
        let mut parent_lines = Vec::new();
        for parent_id in commit.parent_ids() {
            parent_lines.push(
                Line::from(vec![
                    Span::styled(format!("{}", parent_id), Style::default().fg(COLOR_TEXT))
                ])
            );
        }

        // Build commit info text
        let mut commit_lines = vec![
            Line::from(vec![
                Span::styled("Commit sha:", Style::default().fg(COLOR_GREY_400))
            ]),
            Line::from(vec![
                Span::styled(format!("{}", self.shas.get(self.selected).unwrap()), Style::default().fg(COLOR_TEXT))
            ]),
            Line::from(vec![
                Span::styled("Parents:", Style::default().fg(COLOR_GREY_400))
            ])
        ];

        // Insert parent lines
        commit_lines.extend(parent_lines);

        // Add the rest of the commit info
        commit_lines.extend(vec![
            Line::from(vec![
                Span::styled("Authored by: ", Style::default().fg(COLOR_GREY_400)),
            ]),
            Line::from(vec![
                Span::styled(format!("{} {}", author.name().unwrap_or(""), author.email().unwrap_or("")), Style::default().fg(COLOR_TEXT)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}", timestamp_to_utc(author.when())), Style::default().fg(COLOR_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Commited by: ", Style::default().fg(COLOR_GREY_400)),
            ]),
            Line::from(vec![
                Span::styled(format!("{} {}", committer.name().unwrap_or(""), committer.email().unwrap_or("")), Style::default().fg(COLOR_TEXT)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}", timestamp_to_utc(committer.when())), Style::default().fg(COLOR_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Message: ", Style::default().fg(COLOR_GREY_400)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}", message.unwrap_or("")), Style::default().fg(COLOR_TEXT)),
            ]),
        ]);

        let commit_info = Text::from(commit_lines);
        let padding = Padding { left: 1, right: 1, top: 0, bottom: 0 };

        ratatui::widgets::Paragraph::new(commit_info)
            .left_aligned()
            .wrap(Wrap { trim: false })
            .block(ratatui::widgets::Block::default()
                .title(vec![
                    Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    Span::styled("[ Inspector ]", Style::default().fg(COLOR_GREY_400)),
                    Span::styled("─", Style::default().fg(COLOR_BORDER))
                ])
                .title_alignment(ratatui::layout::Alignment::Right)
                .title_style(Style::default().fg(COLOR_GREY_400))
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(COLOR_BORDER))
                .padding(padding)
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks_inspector[0], buf);


        ratatui::widgets::Paragraph::new(get_changed_filenames_text(&self.repo, *self.shas.get(self.selected).unwrap()))
            .left_aligned()
            .wrap(Wrap { trim: false })
            .block(ratatui::widgets::Block::default()
                .title(vec![
                    Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    Span::styled("[ Files ]", Style::default().fg(COLOR_GREY_400)),
                    Span::styled("─", Style::default().fg(COLOR_BORDER))
                ])
                .title_alignment(ratatui::layout::Alignment::Right)
                .title_style(Style::default().fg(COLOR_GREY_400))
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(COLOR_BORDER))
                .padding(padding)
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks_inspector[1], buf);
    }
}


fn timestamp_to_utc(time: Time) -> String {
    // Create a DateTime with the given offset
    let offset = FixedOffset::east_opt(time.offset_minutes() * 60).unwrap();
    
    // Create UTC datetime from timestamp
    let utc_datetime = DateTime::from_timestamp(time.seconds(), 0)
        .expect("Invalid timestamp");
    
    // Convert to local time with offset, then back to UTC
    let local_datetime = offset.from_utc_datetime(&utc_datetime.naive_utc());
    let final_utc: DateTime<Utc> = local_datetime.with_timezone(&Utc);
    
    // Format as string
    final_utc.to_rfc2822()
}

fn get_changed_filenames_text(repo: &Repository, oid: Oid) -> Text<'_> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();

    let mut lines = Vec::new();

    if commit.parent_count() == 0 {
        // Initial commit — list all files
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                lines.push(Line::from(Span::styled(
                    name.to_string(),
                    Style::default().fg(COLOR_GREY_400),
                )));
            }
            git2::TreeWalkResult::Ok
        }).unwrap();
    } else {
        // Normal commits — diff against first parent
        let parent = commit.parent(0).unwrap();
        let parent_tree = parent.tree().unwrap();
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None).unwrap();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    lines.push(Line::from(Span::styled(
                        path.display().to_string(),
                        Style::default().fg(COLOR_GREY_400),
                    )));
                }
                true
            },
            None,
            None,
            None,
        ).unwrap();
    }

    Text::from(lines)
}