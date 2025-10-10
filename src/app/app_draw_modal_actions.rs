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
        Clear,
        Paragraph,
        Widget
    },
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App
};

impl App {

    pub fn draw_modal_actions(&mut self, frame: &mut Frame) {
        
        let length = 60;
        let height = 9;
        let mut lines: Vec<Line> = Vec::new();

        if self.graph_selected == 0 {
            lines = vec![
                Line::from(vec![
                    Span::styled("uncommited changes", Style::default().fg(COLOR_TEXT)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("select an operation to perform"), Style::default().fg(COLOR_TEXT))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("(c)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("ommit "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(r)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("eset "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(a)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("dd "), Style::default().fg(COLOR_TEXT))
                ]),
            ]; 
        } else {
            let oid = *self.oids.get(self.graph_selected).unwrap();      
            lines = vec![
                Line::from(vec![
                    Span::styled("you are here: ", Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("#{:.6}", oid), Style::default().fg(*self.oid_colors.get(&oid).unwrap()))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("select an operation to perform"), Style::default().fg(COLOR_TEXT))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("(c)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("heckout "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(n)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("ew "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(r)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("eset "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(d)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("elete "), Style::default().fg(COLOR_TEXT))
                ]),
            ]; 
        } 
            
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
            .title(Span::styled(" (x) ", Style::default().fg(COLOR_GREY_500)))
            .title_alignment(Alignment::Right)
            .padding(padding)
            .border_type(ratatui::widgets::BorderType::Rounded);

        // Modal content

        let paragraph = Paragraph::new(Text::from(lines))
            .block(modal_block)
            .alignment(Alignment::Center);

        paragraph.render(modal_area, frame.buffer_mut());
    }
}
