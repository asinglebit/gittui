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
            if self.uncommitted.is_clean {
                lines = vec![
                    Line::default(),
                    Line::default(),
                    Line::from(vec![
                        Span::styled(format!("all is up-to-date"), Style::default().fg(COLOR_TEXT))
                    ]),
                    Line::from(vec![
                        Span::styled(format!("(r)"), Style::default().fg(COLOR_GREY_500)),
                        Span::styled(format!("eload"), Style::default().fg(COLOR_TEXT)),
                    ]),
                ];
            } else {
                let mut line_status = Line::default();

                if self.uncommitted.is_staged {
                    line_status.extend(vec![
                        Span::styled("staged: ", Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.staged.modified.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("~", Style::default().fg(COLOR_BLUE)),
                        Span::styled(format!("{} ", self.uncommitted.staged.modified.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.staged.added.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("+", Style::default().fg(COLOR_GREEN)),
                        Span::styled(format!("{} ", self.uncommitted.staged.added.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.staged.deleted.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("-", Style::default().fg(COLOR_RED)),
                        Span::styled(format!("{} ", self.uncommitted.staged.deleted.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.is_staged && self.uncommitted.is_unstaged {
                    line_status.extend(vec![
                        Span::styled("| ", Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.is_unstaged {
                    line_status.extend(vec![
                        Span::styled("unstaged: ", Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.unstaged.modified.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("~", Style::default().fg(COLOR_BLUE)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.modified.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.unstaged.added.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("+", Style::default().fg(COLOR_GREEN)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.added.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }
                if self.uncommitted.unstaged.deleted.len() > 0 {
                    line_status.extend(vec![
                        Span::styled("-", Style::default().fg(COLOR_RED)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.deleted.len()), Style::default().fg(COLOR_TEXT)),
                    ].into_iter());
                }

                let mut line_operations = Line::default();
                if self.uncommitted.is_staged {
                    line_operations.push_span(Span::styled("(c)", Style::default().fg(COLOR_GREY_500)));
                    line_operations.push_span(Span::styled("ommit ", Style::default().fg(COLOR_TEXT)));
                    line_operations.push_span(Span::styled("(u)", Style::default().fg(COLOR_GREY_500)));
                    line_operations.push_span(Span::styled("nstage ", Style::default().fg(COLOR_TEXT)));
                }
                if self.uncommitted.is_unstaged {
                    line_operations.push_span(Span::styled("(a)", Style::default().fg(COLOR_GREY_500)));
                    line_operations.push_span(Span::styled("dd ", Style::default().fg(COLOR_TEXT)));
                }

                line_operations.push_span(Span::styled("(r)", Style::default().fg(COLOR_GREY_500)));
                line_operations.push_span(Span::styled("eload ", Style::default().fg(COLOR_TEXT)));

                lines = vec![
                    line_status,
                    Line::from(""),
                    Line::from(vec![Span::styled("select an operation to perform", Style::default().fg(COLOR_TEXT))]),
                    Line::from(""),
                    line_operations,
                ];
            }
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
                    Span::styled(format!("(h)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("ardreset "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(m)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("ixedreset "), Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("(r)"), Style::default().fg(COLOR_GREY_500)),
                    Span::styled(format!("eload"), Style::default().fg(COLOR_TEXT)),
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
