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
    helpers::{
        palette::*
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App
};

impl App {

    pub fn draw_modal_checkout(&mut self, frame: &mut Frame) {
        
        let oid = *self.oids.get(self.graph_selected).unwrap();
        let color = self.tip_colors.get(&oid).unwrap();
        let mut length = 39;
        let branches = self
            .tips
            .entry(oid)
            .or_default();
        let mut lines = vec![
            Line::from(vec![
                Span::styled("select a branch to checkout".to_string(), Style::default().fg(COLOR_TEXT))
            ]),
            Line::from("")
        ];
        branches.iter().enumerate().for_each(|(idx, branch_name)| {
            length = (10 + branch_name.len()).max(length);
            lines.push(Line::from(Span::styled(
                format!("● {} ", branch_name),
                Style::default().fg(if idx == self.modal_checkout_selected as usize { *color } else { COLOR_TEXT }),
            )));
        });
        let height = branches.len() + 6;

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

        paragraph.render(modal_area, frame.buffer_mut());
    }
}
