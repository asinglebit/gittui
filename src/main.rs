use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use git2::{Repository};
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    counter: i32,
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
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let repo = Repository::open(".").expect("msg");
        let mut revwalk = repo.revwalk().expect("msg");
        let _ = revwalk.push_head(); // Start from HEAD

        // Iterate over the commit IDs
        for oid_result in revwalk {
            let oid = oid_result.expect("msg");
            let commit = repo.find_commit(oid).expect("msg");

            println!(
                "commit {}\nAuthor: {}\nDate: {:?}\n\n    {}\n",
                commit.id(),
                commit.author(),
                commit.time(),
                commit.message().unwrap_or("<no message>")
            );
        }

        // Split the widget's area into two vertical chunks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // top half
                Constraint::Percentage(50), // bottom half
            ])
            .split(area);

        // Top half: counter block
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(chunks[0], buf); // render into top chunk

        // Bottom half: placeholder panel
        let bottom_text = Paragraph::new("This is the bottom half")
            .centered()
            .block(Block::bordered().title("Bottom Panel"));
        bottom_text.render(chunks[1], buf); // render into bottom chunk
    }
}