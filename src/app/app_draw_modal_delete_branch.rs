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
    git::{
        queries::{
            commits::{
                get_current_branch
            }
        }
    }
};

impl App {

    pub fn draw_modal_delete_branch(&mut self, frame: &mut Frame) {
        
        let current = get_current_branch(&self.repo);
        let oid = *self.oids.get(self.graph_selected).unwrap();
        let color = self.tip_colors.get(&oid).unwrap();
        
        let mut lines = Vec::new();
        let mut length = 25;

        // Static lines
        if let Some(branch) = &current {
            length = length.max(2 + branch.len());
            lines.push(Line::from(vec![
                Span::styled("you are on a branch", Style::default().fg(self.theme.COLOR_TEXT)),
            ]));
        lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled(format!("● {}", branch), Style::default().fg(self.theme.COLOR_GRASS)),
            ]));
        } else {
            let oid = self.repo.head().unwrap().target().unwrap();
            length = length.max(26);
            lines.push(Line::from(vec![
                Span::styled("you are on a detached head", Style::default().fg(self.theme.COLOR_TEXT)),
            ]));
        lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled(format!("#{:.6}", oid), Style::default().fg(self.theme.COLOR_GRASS)),
            ]));
        }

        // Second static line
        let line_text = "select a branch to delete";
        lines.push(Line::default());
        lines.push(Line::from(vec![Span::styled(line_text, Style::default().fg(self.theme.COLOR_TEXT))]));

        // Empty line
        lines.push(Line::default());
            
        let mut height = 10;
        let branches = self.visible_branches.get(&oid).unwrap();

        branches
            .iter()
            .filter(|branch| current.as_ref().map_or(true, |c| c != *branch))
            .enumerate()
            .for_each(|(idx, branch)| {
                height += 1;
                let is_local = self
                    .tips_local
                    .values()
                    .any(|branches| branches.iter().any(|b| b.as_str() == branch));

                let line_text = format!("{} {} ", if is_local { "●" } else { "◆" }, branch);
                length = length.max(line_text.len());

                lines.push(Line::from(Span::styled(
                    line_text,
                    Style::default().fg(if idx == self.modal_delete_branch_selected as usize {
                        *color
                    } else {
                        self.theme.COLOR_TEXT
                    }),
                )));
            });

        let bg_block = Block::default().style(Style::default().fg(self.theme.COLOR_BORDER));
        bg_block.render(frame.area(), frame.buffer_mut());

        // Modal size (smaller than area)
        length += 10;
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
            .border_style(Style::default().fg(self.theme.COLOR_GREY_600))
            .title(Span::styled(" (esc) ", Style::default().fg(self.theme.COLOR_GREY_500)))
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
