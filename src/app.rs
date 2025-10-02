use std::{cell::Cell, io};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer, layout::Rect, text::{Line, Span, Text}, widgets::{Borders, Widget}, DefaultTerminal, Frame
};
use ratatui::style::{Style, Color};

use crate::helpers::get_commits;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    scroll: Cell<u16>,
    selected: usize,
    structure: Vec<Line<'static>>,
    descriptors: Vec<Line<'static>>,
    messages: Vec<Line<'static>>,
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
         
            KeyCode::Char('j') | KeyCode::Down => { if self.selected + 1 < self.descriptors.len() { self.selected += 1; } }
            KeyCode::Char('k') | KeyCode::Up => { if self.selected > 0 { self.selected -= 1; } }
            _ => {}
        }
    }

    fn reload(&mut self) {
        let (structure, descriptors, messages) = get_commits();
        self.structure = structure;
        self.descriptors = descriptors;
        self.messages = messages;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(15), ratatui::layout::Constraint::Percentage(15), ratatui::layout::Constraint::Percentage(70)])
            .split(area);

        let viewport_height = chunks[0].height as usize - 2;

        // Clamp scroll so selected line is visible
        if self.selected < self.scroll.get() as usize {
            self.scroll.set(self.selected as u16);
        } else if self.selected >= self.scroll.get() as usize + viewport_height {
            self.scroll.set((self.selected + 1 - viewport_height) as u16);
        }

        let width = chunks[0].width as usize;
        let mut structure_lines: Vec<Line> = self.structure
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();

                    // Pad with spaces to fill the width
                    if line_content.len() < width {
                        line_content.push_str(&" ".repeat(width));
                    }

                    Line::from(Span::styled(
                        line_content,
                        Style::default().bg(Color::Blue).fg(Color::White),
                    ))
                } else {
                    line.clone()
                }
            })
            .collect();
        let structure_text = Text::from(structure_lines);

        let width = chunks[1].width as usize;
        let mut descriptors_lines: Vec<Line> = self.descriptors
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();

                    // Pad with spaces to fill the width
                    if line_content.len() < width {
                        line_content.push_str(&" ".repeat(width - line_content.len()));
                    }

                    Line::from(Span::styled(
                        line_content,
                        Style::default().bg(Color::Blue).fg(Color::White),
                    ))
                } else {
                    line.clone()
                }
            })
            .collect();
        let descriptors_text = Text::from(descriptors_lines);

        let width = chunks[2].width as usize;
        let mut messages_lines: Vec<Line> = self.messages
            .iter()
            .enumerate()
            .map(|(i, line)| {
                if i == self.selected {
                    let content = line.to_string();
                    let mut line_content = content.clone();

                    // Pad with spaces to fill the width
                    if line_content.len() < width {
                        line_content.push_str(&" ".repeat(width - line_content.len()));
                    }

                    Line::from(Span::styled(
                        line_content,
                        Style::default().bg(Color::Blue).fg(Color::White),
                    ))
                } else {
                    line.clone()
                }
            })
            .collect();
        let messages_text = Text::from(messages_lines);

        ratatui::widgets::Paragraph::new(structure_text.clone())
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[0], buf);

        ratatui::widgets::Paragraph::new(descriptors_text)
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[1], buf);

        ratatui::widgets::Paragraph::new(messages_text)
            .left_aligned()
            .scroll((self.scroll.get(), 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[2], buf);
    }
}
