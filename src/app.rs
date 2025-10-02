use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Borders, Widget},
    DefaultTerminal, Frame,
};
use ratatui::style::{Style, Color};

use crate::helpers::get_commits;

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    scroll: u16
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
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => self.exit(),
            KeyCode::Char('j') | KeyCode::Down => self.scroll += 5,
            KeyCode::Char('k') | KeyCode::Up => self.scroll -= if self.scroll > 4 { 5 } else { 0 },
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (structure, descriptors, messages) = get_commits();
        let structure_text = ratatui::text::Text::from(structure);
        let descriptors_text = ratatui::text::Text::from(descriptors);
        let messages_text = ratatui::text::Text::from(messages);

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(15), ratatui::layout::Constraint::Percentage(15), ratatui::layout::Constraint::Percentage(70)])
            .split(area);

        ratatui::widgets::Paragraph::new(structure_text.clone())
            .left_aligned()
            .scroll((self.scroll, 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[0], buf);

        ratatui::widgets::Paragraph::new(descriptors_text)
            .left_aligned()
            .scroll((self.scroll, 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[1], buf);

        ratatui::widgets::Paragraph::new(messages_text)
            .left_aligned()
            .scroll((self.scroll, 0))
            .block(ratatui::widgets::Block::default()
                .borders(Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb((60), (60), (60))))
                .border_type(ratatui::widgets::BorderType::Rounded))
            .render(chunks[2], buf);
    }
}
