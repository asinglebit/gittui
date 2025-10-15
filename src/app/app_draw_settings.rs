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
        Borders,
        Scrollbar,
        ScrollbarOrientation,
        ScrollbarState,
        List,
        ListItem
    }
};
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

        // Get vertical dimensions
        let total_lines = self.viewer_lines.len();
        let visible_height = self.layout.graph.height as usize - 2;

        // Clamp selection
        if total_lines == 0 {
            self.viewer_selected = 0;
        } else if self.viewer_selected >= total_lines {
            self.viewer_selected = total_lines - 1;
        }
        
        // Trap selection
        self.trap_selection(self.viewer_selected, &self.viewer_scroll, total_lines, visible_height);

        // Calculate scroll
        let start = self.viewer_scroll.get().min(total_lines.saturating_sub(visible_height));
        let end = (start + visible_height).min(total_lines);

        let a = get_git_user_info(&self.repo).unwrap();
        // Setup list items
        let mut list_items: Vec<ListItem> = Vec::new();

        let dummies = if self.layout.app.width < 80 {
                (visible_height / 2).saturating_sub((8 + 1 - 3) / 2)
            } else if self.layout.app.width < 120 {
                (visible_height / 2).saturating_sub((8 + 9 - 3) / 2)
            } else {
                (visible_height / 2).saturating_sub((8 + 11 - 3) / 2)
            };

        for idx in 0..dummies { list_items.push(ListItem::from(Line::default())); }
        
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(center_line(format!("name: {}", a.0.unwrap() ).as_str(), max_text_width), Style::default().fg(COLOR_TEXT))
        ])));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(center_line(format!("email: {}", a.1.unwrap()).as_str(), max_text_width), Style::default().fg(COLOR_TEXT))
        ])));

        list_items.push(ListItem::from(Line::default()));
        list_items.push(ListItem::from(Line::default()));

        if self.layout.app.width < 80 {
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("guita╭").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));
        } else if self.layout.app.width < 120 {
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("                  :#   :#                   ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("                       L#                   ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!(".##5#^.  .#   .#  :C  #C6#   #?##:          ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("#B   #G  C#   #B  #7   B?        G#         ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("#4   B5  B5   B5  B5   B5    1B5B#G  .a###  ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("#b   5?  ?B   B5  B5   B5   ##   ##  B?     ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!(".#B~6G!  .#6#~G.  #5   ~##  .##Y~#.  !#     ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("    .##                              !B     ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("   ~G#                               ~?     ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));
        } else {
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("                                :GG~        .?Y.                                ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("      ....        ..      ..   .....      . ^BG: ..       .....                 ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("   .7555YY7JP^   ~PJ     ~PJ  ?YY5PP~    7YY5BGYYYYJ.   J555YY557.              ").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("  .5B?.  :JBB~   !#5     !#5  ...PB~     ...^BG:....    ~:.   .7#5           :^^").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("  7#5     .GB~   !B5     !B5     PB~        :BG.        .~7??J?JBG:      .~JPPPY").as_str(), max_text_width), Style::default().fg(COLOR_GRASS)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("  ?#Y      PB~   !B5     !B5     PB~        :BG.       7GP7~^^^!BG:     ~5GY!:. ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("  ^GB~    7BB~   ^BG.   .YB5     5#7        :BB:       P#!     JBG:    ^GG7     ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("   ^5G5JJYJPB~    JBP???YYB5     ^5GYJJ?.    7GPJ???.  ~PGJ77?5J5B!    JG5      ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("     .^~^..GB:     :~!!~. ^^       :~~~~      .^~~~~    .^!!!~. .^:    JG5      ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("   .?!^^^!5G7                                                          YB5      ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
            list_items.push(ListItem::from(Line::from(Span::styled(center_line(format!("   .!?JJJ?!:                                                           75?      ").as_str(), max_text_width), Style::default().fg(COLOR_GREEN)))));    
        }
        list_items.push(ListItem::from(Line::default()));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(center_line(format!("please make sure to have your ssh agent running").as_str(), max_text_width), Style::default().fg(COLOR_TEXT))
        ])));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(center_line(format!("press (esc) to exit to graph, use (tab) and (arrow)s to navigate").as_str(), max_text_width), Style::default().fg(COLOR_TEXT))
        ])));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(center_line(format!("press (enter) for menus, (i) for inspector, (s) for status").as_str(), max_text_width), Style::default().fg(COLOR_TEXT))
        ])));

        // Setup the list
        let list = List::new(list_items)
            .block(
                Block::default()
                    .padding(padding)
            );

        // Render the list
        frame.render_widget(list, self.layout.app);

        // Setup the scrollbar
        let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(self.viewer_scroll.get());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╮") })
            .end_symbol(if self.is_inspector || self.is_status { Some("─") } else { Some("╯") })
            .track_symbol(Some("│"))
            .thumb_symbol("▌")
            .thumb_style(Style::default().fg(if self.focus == Focus::Viewport {
                COLOR_GREY_600
            } else {
                COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.graph, &mut scrollbar_state);
    }
}
