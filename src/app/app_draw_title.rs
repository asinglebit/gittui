#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Line,
        Span,
        Text
    },
    widgets::{
        Block,
    },
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*,
    },
};
#[rustfmt::skip]
use crate::app::{
    app::App,
};

impl App {

    pub fn draw_title(&mut self, frame: &mut Frame) {

        let path = if let Some(file_name) = self.file_name.clone() { format!("{}/{}", self.path.clone(), file_name) } else { self.path.clone() };

        let paragraph = ratatui::widgets::Paragraph::new(Text::from(Line::from([
            self.logo.clone(), vec![
            Span::styled(" |", Style::default().fg(COLOR_TEXT)),
            Span::styled(format!(" 🖿  {}", path), Style::default().fg(COLOR_TEXT)),
            
        ]].concat())))
        .left_aligned()
        .block(Block::default());
        frame.render_widget(paragraph, self.layout.title_left);
    }
}
