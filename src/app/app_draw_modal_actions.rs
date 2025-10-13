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
    app::app::{
        App
    },
    helpers::{
        palette::*
    },
};

impl App {

    pub fn draw_modal_actions(&mut self, frame: &mut Frame) {
        
        let length = 60;
        let mut height = 9;

        #[allow(unused_assignments)] // This is a bug in clippy
        let mut lines: Vec<Line> = Vec::new();

        if self.graph_selected == 0 {
            if self.uncommitted.is_clean {
                lines = vec![
                    Line::default(),
                    Line::from(vec![
                        Span::styled("all is up-to-date".to_string(), Style::default().fg(COLOR_TEXT))
                    ]),
                    Line::default(),
                    Line::from(vec![
                        Span::styled("(r)".to_string(), Style::default().fg(COLOR_GREY_500)),
                        Span::styled("eload".to_string(), Style::default().fg(COLOR_TEXT)),
                    ]),
                ];
            } else {
                let mut line_status = Line::default();

                if self.uncommitted.is_staged {
                    line_status.extend(vec![
                        Span::styled("staged: ", Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.staged.modified.is_empty() {
                    line_status.extend(vec![
                        Span::styled("~", Style::default().fg(COLOR_BLUE)),
                        Span::styled(format!("{} ", self.uncommitted.staged.modified.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.staged.added.is_empty() {
                    line_status.extend(vec![
                        Span::styled("+", Style::default().fg(COLOR_GREEN)),
                        Span::styled(format!("{} ", self.uncommitted.staged.added.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.staged.deleted.is_empty() {
                    line_status.extend(vec![
                        Span::styled("-", Style::default().fg(COLOR_RED)),
                        Span::styled(format!("{} ", self.uncommitted.staged.deleted.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if self.uncommitted.is_staged && self.uncommitted.is_unstaged {
                    line_status.extend(vec![
                        Span::styled("| ", Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if self.uncommitted.is_unstaged {
                    line_status.extend(vec![
                        Span::styled("unstaged: ", Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.unstaged.modified.is_empty() {
                    line_status.extend(vec![
                        Span::styled("~", Style::default().fg(COLOR_BLUE)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.modified.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.unstaged.added.is_empty() {
                    line_status.extend(vec![
                        Span::styled("+", Style::default().fg(COLOR_GREEN)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.added.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
                }
                if !self.uncommitted.unstaged.deleted.is_empty() {
                    line_status.extend(vec![
                        Span::styled("-", Style::default().fg(COLOR_RED)),
                        Span::styled(format!("{} ", self.uncommitted.unstaged.deleted.len()), Style::default().fg(COLOR_TEXT)),
                    ]);
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

                line_operations.push_span(Span::styled("(f)", Style::default().fg(COLOR_GREY_500)));
                line_operations.push_span(Span::styled("etch ", Style::default().fg(COLOR_TEXT)));
                line_operations.push_span(Span::styled("(p)", Style::default().fg(COLOR_GREY_500)));
                line_operations.push_span(Span::styled("ushforce ", Style::default().fg(COLOR_TEXT)));
                line_operations.push_span(Span::styled("(r)", Style::default().fg(COLOR_GREY_500)));
                line_operations.push_span(Span::styled("eload ", Style::default().fg(COLOR_TEXT)));

                lines = vec![
                    line_status,
                    Line::default(),
                    Line::from(vec![Span::styled("select an operation to perform", Style::default().fg(COLOR_TEXT))]),
                    Line::default(),
                    line_operations,
                ];
            }
        } else {
            height = 11;
            let oid = *self.oids.get(self.graph_selected).unwrap();      
            lines = vec![
                Line::from(vec![
                    Span::styled("you are here: ", Style::default().fg(COLOR_TEXT)),
                    Span::styled(format!("#{:.6}", oid), Style::default().fg(*self.oid_colors.get(&oid).unwrap()))
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("select an operation to perform".to_string(), Style::default().fg(COLOR_TEXT))
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("(c)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("heckout ".to_string(), Style::default().fg(COLOR_TEXT)),
                    Span::styled("(h)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("ardreset ".to_string(), Style::default().fg(COLOR_TEXT)),
                    Span::styled("(m)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("ixedreset ".to_string(), Style::default().fg(COLOR_TEXT)),
                    Span::styled("(f)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("etch ".to_string(), Style::default().fg(COLOR_TEXT)),
                ]),
                Line::default(),
                Line::from(vec![
                    Span::styled("(p)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("ushforce ".to_string(), Style::default().fg(COLOR_TEXT)),
                    Span::styled("(r)".to_string(), Style::default().fg(COLOR_GREY_500)),
                    Span::styled("eload".to_string(), Style::default().fg(COLOR_TEXT)),
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
