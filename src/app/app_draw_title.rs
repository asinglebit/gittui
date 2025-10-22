#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Line,
        Span
    },
    widgets::{
        Block,
    },
};
#[rustfmt::skip]
use crate::app::{
    app::App,
};

impl App {

    pub fn draw_title(&mut self, frame: &mut Frame) {

        // Logo and path
        let path = if let Some(file_name) = self.file_name.clone() { format!("{}/{}", self.path.clone(), file_name) } else { self.path.clone() };
        let logo = self.logo.clone();
        let separator = Span::styled(" |", Style::default().fg(self.theme.COLOR_TEXT));
        let folder = Span::styled(format!(" 🖿  {}", path), Style::default().fg(self.theme.COLOR_TEXT));
        let line = Line::from([logo, vec![ separator, folder ]].concat());
        let paragraph = ratatui::widgets::Paragraph::new(line)
            .left_aligned()
            .block(Block::default());
        frame.render_widget(paragraph, self.layout.title_left);

        // Hint
        let hint = Span::styled(format!("{} ", self.hint), Style::default().fg(self.theme.COLOR_GRASS));
        let line = Line::from(vec![hint]);
        let paragraph = ratatui::widgets::Paragraph::new(line)
            .right_aligned()
            .block(Block::default());
        frame.render_widget(paragraph, self.layout.title_right);
    }
}
