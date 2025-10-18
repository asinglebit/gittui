#[rustfmt::skip]
use ratatui::{
    Frame,
    style::Style,
    text::{
        Span,
        Line
    },
    widgets::{
        Block,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        List,
        ListItem,
    }
};
use crate::helpers::text::fill_width;
#[rustfmt::skip]
use crate::{
    helpers::{
        palette::*
    },
};
#[rustfmt::skip]
use crate::{
    app::app::{
        App,
        Focus,
    },
    git::{
        queries::{
            commits::{
                get_git_user_info
            }
        }
    },
    helpers::{
        text::{
            center_line
        }
    },
    core::{
        renderers::{
            render_keybindings
        }
    }
};

impl App {

    pub fn draw_settings(&mut self, frame: &mut Frame) {
        
        // Padding
        let padding = ratatui::widgets::Padding {
            left: 1,
            right: 1,
            top: 0,
            bottom: 0,
        };
        
        // Calculate maximum available width for text
        let available_width = self.layout.graph.width as usize - 1;
        let max_text_width = available_width.saturating_sub(2);

        // Credentials
        let (name, email) = get_git_user_info(&self.repo).unwrap();

        // Setup list items
        let mut lines: Vec<Line> = Vec::new();

        let mut logo_height = 0;

        lines.push(Line::default());
        if self.layout.app.width < 80 {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(format!("guita╭"), Style::default().fg(COLOR_GRASS))).centered());
            lines.push(Line::default());
            logo_height = 3;
        } else if self.layout.app.width < 120 && self.layout.app.height > 24 {
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(format!("                    :#   :#                 "), Style::default().fg(COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled(format!("                         L#                 "), Style::default().fg(COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled(format!("  .##5#^.  .#   .#  :C  #C6#   #?##:        "), Style::default().fg(COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled(format!("  #B   #G  C#   #B  #7   B?        G#       "), Style::default().fg(COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled(format!("  #4   B5  B5   B5  B5   B5    1B5B#G  .a###"), Style::default().fg(COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled(format!("  #b   5?  ?B   B5  B5   B5   ##   ##  B?   "), Style::default().fg(COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled(format!("  .#B~6G!  .#6#~G.  #5   ~##  .##Y~#.  !#   "), Style::default().fg(COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled(format!("      .##                              !B   "), Style::default().fg(COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled(format!("     ~G#                               ~?   "), Style::default().fg(COLOR_GREEN))).centered());
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
            logo_height = 15;
        } else if self.layout.app.height > 30{
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(format!("                                 :GG~        .?Y.                                "), Style::default().fg(COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled(format!("       ....        ..      ..   .....      . ^BG: ..       .....                 "), Style::default().fg(COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled(format!("    .7555YY7JP^   ~PJ     ~PJ  ?YY5PP~    7YY5BGYYYYJ.   J555YY557.              "), Style::default().fg(COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled(format!("   .5B?.  :JBB~   !#5     !#5  ...PB~     ...^BG:....    ~:.   .7#5           :^^"), Style::default().fg(COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled(format!("   7#5     .GB~   !B5     !B5     PB~        :BG.        .~7??J?JBG:      .~JPPPY"), Style::default().fg(COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled(format!("   ?#Y      PB~   !B5     !B5     PB~        :BG.       7GP7~^^^!BG:     ~5GY!:. "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled(format!("   ^GB~    7BB~   ^BG.   .YB5     5#7        :BB:       P#!     JBG:    ^GG7     "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled(format!("    ^5G5JJYJPB~    JBP???YYB5     ^5GYJJ?.    7GPJ???.  ~PGJ77?5J5B!    JG5      "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled(format!("      .^~^..GB:     :~!!~. ^^       :~~~~      .^~~~~    .^!!!~. .^:    JG5      "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled(format!("    .?!^^^!5G7                                                          YB5      "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled(format!("    .!?JJJ?!:                                                           75?      "), Style::default().fg(COLOR_GREEN))).centered());    
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
            logo_height = 17;
        }
        lines.push(Line::from(vec![
            Span::styled(fill_width("credentials", "", max_text_width / 2), Style::default().fg(COLOR_TEXT))
        ]).centered());
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled(fill_width("name:", name.unwrap().as_str(), max_text_width / 2), Style::default().fg(COLOR_TEXT).bg(COLOR_GREY_900))
        ]).centered());
        lines.push(Line::from(vec![
            Span::styled(fill_width("email:", email.unwrap().as_str(), max_text_width / 2), Style::default().fg(COLOR_TEXT))
        ]).centered());
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled(fill_width("key bindings:", "", max_text_width / 2), Style::default().fg(COLOR_TEXT))
        ]).centered());
        lines.push(Line::default());
        render_keybindings(&self.keymap, max_text_width / 2).iter().enumerate().for_each(|(idx, kb_line)| {
            let spans: Vec<Span> = kb_line.clone().spans.iter().map(|span| {
                let mut style = span.style;
                if idx % 2 == 0 { style = style.bg(COLOR_GREY_900); }
                Span::styled(span.content.clone(), style)
            }).collect();
            lines.push(Line::from(spans).centered());
        });

        // Get vertical dimensions
        let total_lines = lines.len();
        let visible_height = self.layout.graph.height as usize;

        // Clamp selection
        let upper_limit = logo_height + 8;
        if total_lines == 0 {
            self.settings_selected = 0;
        } else if self.settings_selected >= total_lines {
            self.settings_selected = total_lines - 1;
        } else if self.settings_selected < upper_limit {
            self.settings_selected = upper_limit;
        }
        
        // Calculate sticky scroll
        let start = if self.settings_selected + 1 > visible_height { self.settings_selected + 1 - visible_height } else { 0 };
        let end = (start + visible_height).min(total_lines);

        // Setup list items
        let list_items: Vec<ListItem> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let absolute_idx = start + i;
                let mut item = line.clone();
                if absolute_idx == self.settings_selected && self.focus == Focus::Viewport {
                    let spans: Vec<Span> = item.clone().spans.iter().map(|span| {
                        let mut style = span.style;
                        style = style.bg(COLOR_GREY_800);
                        Span::styled(span.content.clone(), style)
                    }).collect();
                    item = Line::from(spans).centered();
                }
                ListItem::from(item)
            })
            .collect();

        // Setup the list
        let list = List::new(list_items)
            .block(
                Block::default()
                    .padding(padding)
            );

        // Render the list
        frame.render_widget(list, self.layout.graph);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(start);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("╮"))
            .end_symbol(Some("╯"))
            .track_symbol(Some("│"))
            .thumb_symbol("▌")
            .thumb_style(Style::default().fg(if self.focus == Focus::Viewport {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.app, &mut scrollbar_state);
    }
}
