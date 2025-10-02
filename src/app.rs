use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Widget},
    DefaultTerminal, Frame,
};

use crate::helpers::get_commits;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (structure, descriptors) = get_commits();
        let structure_text = ratatui::text::Text::from(structure);
        let descriptors_text = ratatui::text::Text::from(descriptors);

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(20), ratatui::layout::Constraint::Percentage(80)])
            .split(area);

        ratatui::widgets::Paragraph::new(structure_text.clone())
            .left_aligned()
            .block(ratatui::widgets::Block::bordered())
            .render(chunks[0], buf);

        ratatui::widgets::Paragraph::new(descriptors_text)
            .left_aligned()
            .block(ratatui::widgets::Block::bordered())
            .render(chunks[1], buf);
    }
}
