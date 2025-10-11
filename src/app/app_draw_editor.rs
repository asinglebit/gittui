#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::Span,
    widgets::{
        Block,
        Widget,
        Borders,
        Padding,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
    },
};
#[rustfmt::skip]
use edtui::{
    EditorStatusLine,
    EditorTheme,
    EditorView,
    SyntaxHighlighter
};
#[rustfmt::skip]
use crate::{
    utils::{
        colors::*
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Focus
};

impl App {

    pub fn draw_editor(&mut self, frame: &mut Frame) {

        let height = self.layout.graph.height as u16 - 2;
        let width = self.layout.graph.width as u16 - 2;

        // Modal block
        Block::default()
            .title(vec![
                Span::styled("─", Style::default().fg(COLOR_BORDER)),
                Span::styled(" editor ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                Span::styled("─", Style::default().fg(COLOR_BORDER)),
            ])
            .title_alignment(ratatui::layout::Alignment::Right)
            .title_style(Style::default().fg(COLOR_GREY_400))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_BORDER))
            .render(self.layout.graph, frame.buffer_mut());
        
        // Theme
        let custom_theme = EditorTheme {
            base: Style::default().fg(COLOR_GREY_500),
            cursor_style: Style::default().bg(COLOR_TEXT),
            selection_style: Style::default(),
            block: Some(
                Block::default()
                    .padding(Padding {
                        left: 1,
                        right: 2,
                        top: 0,
                        bottom: 0,
                    })
                    .title(vec![
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                        Span::styled(" editor ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
                        Span::styled("─", Style::default().fg(COLOR_BORDER)),
                    ])
                    .title_alignment(ratatui::layout::Alignment::Right)
                    .title_style(Style::default().fg(COLOR_GREY_400))
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(COLOR_GREY_800))
            ),
            status_line: Some(EditorStatusLine::default()
                .style_text(Style::default().fg(COLOR_TEXT))
                .style_line(Style::default().fg(COLOR_GREY_800))
                .align_left(true)
            )
        };

        // View
        let editor_view = EditorView::new(&mut self.file_editor)
            .theme(custom_theme)
            .wrap(false)
            .syntax_highlighter(Some(SyntaxHighlighter::new("dracula", "json")));
        
        // Render the editor in the modal area
        editor_view.render(self.layout.graph, frame.buffer_mut());

        // Render the scrollbar
        let total_lines = self.oids.len();
        let visible_height = self.layout.graph.height as usize;
        if total_lines > visible_height {
            let mut scrollbar_state = ScrollbarState::new(total_lines).position(self.graph_scroll.get());
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╮") })
                .end_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╯") })
                .track_symbol(Some("│"))
                .thumb_symbol("│")
                .thumb_style(Style::default().fg(if self.focus == Focus::Viewport {
                    COLOR_BORDER
                } else {
                    COLOR_BORDER
                }));

            frame.render_stateful_widget(scrollbar, self.layout.graph, &mut scrollbar_state);
        }
    }
}
