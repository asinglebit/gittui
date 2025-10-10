use ratatui::layout::Position;
#[rustfmt::skip]
use ratatui::{
    Frame,
    prelude::StatefulWidget,
    style::{
        Style,
        Stylize
    },
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
use rat_text::text_input::TextInput;
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

    pub fn draw_modal_commit(&mut self, frame: &mut Frame) {
        
        let mut length = 60;
        let mut height = 9;

        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("commit message:", Style::default().fg(COLOR_TEXT)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("(enter)"), Style::default().fg(COLOR_GREY_500)),
                Span::styled(format!("commit "), Style::default().fg(COLOR_TEXT)),
            ]),
        ];
            
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
            .title(Span::styled(" (esc) ", Style::default().fg(COLOR_GREY_500)))
            .title_alignment(Alignment::Right)
            .padding(padding)
            .border_type(ratatui::widgets::BorderType::Rounded);

        // Modal content
        let paragraph = Paragraph::new(Text::from(lines))
            .block(modal_block)
            .alignment(Alignment::Center);
        
        // Render the paragraph
        paragraph.render(modal_area, frame.buffer_mut());
        
        // Create the input field
        let text_input = TextInput::new()
            .style(Style::default().bg(COLOR_GREY_800))
            .select_style(Style::default().black().on_yellow());
        let input_area = Rect {
            x: modal_area.x + modal_area.width / 2 - 20,
            y: modal_area.y + 4,
            width: 40,
            height: 1,
        };
        text_input.render(input_area, frame.buffer_mut(), &mut self.commit_message_input);

        // Render the cursor
        let position = Position {
            x: input_area.x + self.commit_message_input_cursor as u16,
            y: input_area.y
        };
        frame.set_cursor_position(position);
    }
}
