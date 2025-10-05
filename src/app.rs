use std::{cell::Cell, env, io, path::PathBuf};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use git2::Repository;
use ratatui::{
    buffer::Buffer, layout::Rect, text::{Line, Span, Text}, widgets::{Borders, Widget}, DefaultTerminal, Frame
};
use ratatui::style::{Style, Color};

use crate::helpers::get_commits;

pub struct App {
    // General
    path: String,
    repo: Repository,
    
    // Data
    graph: Vec<Line<'static>>,
    branches: Vec<Line<'static>>,
    messages: Vec<Line<'static>>,
    buffers: Vec<Line<'static>>,
    
    // Interface
    scroll: Cell<u16>,
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
        let (graph, branches, messages, buffer) = get_commits(&self.repo);
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
            KeyCode::Char('j') | KeyCode::Down => { if self.selected + 1 < self.branches.len() { self.selected += 1; } }
            KeyCode::Char('k') | KeyCode::Up => { if self.selected > 0 { self.selected -= 1; } }
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
            .constraints([ratatui::layout::Constraint::Percentage(80), ratatui::layout::Constraint::Percentage(0), ratatui::layout::Constraint::Percentage(20)])
            .split(chunks_vertical[0]);
        let chunks_horizontal = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            // .constraints([ratatui::layout::Constraint::Percentage(15), ratatui::layout::Constraint::Percentage(55), ratatui::layout::Constraint::Percentage(30)])
            .constraints([ratatui::layout::Constraint::Length(15), ratatui::layout::Constraint::Percentage(100), ratatui::layout::Constraint::Length(1)])
            .split(chunks_vertical[1]);

        // Clamp scroll so selected line is visible
        let height = chunks_horizontal[0].height as usize - 2;
        if self.selected < self.scroll.get() as usize {
            self.scroll.set(self.selected as u16);
        } else if self.selected >= self.scroll.get() as usize + height {
            self.scroll.set((self.selected + 1 - height) as u16);
        }

        // Graph
        let width = chunks_horizontal[0].width as usize;
        let graph_lines: Vec<Line> = self.graph
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();
                    line_content.push_str(&" ".repeat(width));
                    Line::from(Span::styled(line_content, Style::default().bg(Color::Blue).fg(Color::Gray)))
                } else {
                    line.clone()
                }
            })
            .collect();
        let graph_text = Text::from(graph_lines);

        // Branches
        let width = chunks_horizontal[1].width as usize;
        let branches_lines: Vec<Line> = self.branches
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();
                    line_content.push_str(&" ".repeat(width));
                    Line::from(Span::styled(line_content, Style::default().bg(Color::Blue).fg(Color::Gray)))
                } else {
                    line.clone()
                }
            })
            .collect();
        let branches_text = Text::from(branches_lines);

        // Commits
        let width = chunks_horizontal[2].width as usize;
        let messages_lines: Vec<Line> = self.buffers
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();
                    line_content.push_str(&" ".repeat(width));
                    Line::from(Span::styled(line_content, Style::default().bg(Color::Blue).fg(Color::Gray)))
                } else {
                    line.clone()
                }
            })
            .collect();
        let messages_text = Text::from(messages_lines);

        // Title
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(format!(" 🖿 {}", self.path), Style::default().fg(Color::Rgb(160, 160, 160))))))
            .left_aligned()
            .block(ratatui::widgets::Block::default())
            .render(chunks_title_bar[0], buf);
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(format!("{}/{} ", self.selected + 1, self.messages.len()), Style::default().fg(Color::Rgb(160, 160, 160))))))
            .right_aligned()
            .block(ratatui::widgets::Block::default())
            .render(chunks_title_bar[2], buf);

        // Status bar
        ratatui::widgets::Paragraph::new(Text::from(Line::from(Span::styled(format!("⟨r⟩ reload ⟨j|k⟩ scroll ⟨q⟩ quit"), Style::default().fg(Color::DarkGray)))))
            .centered()
            .block(ratatui::widgets::Block::default())
            .render(chunks_vertical[2], buf);

        // Paragraphs
        ratatui::widgets::Paragraph::new(graph_text.clone())
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks_horizontal[0], buf);
        ratatui::widgets::Paragraph::new(branches_text)
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks_horizontal[1], buf);
        ratatui::widgets::Paragraph::new(messages_text)
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks_horizontal[2], buf);
    }
}
