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

    pub fn draw_splash(&mut self, frame: &mut Frame) {
        
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
                (visible_height / 2).saturating_sub((3 + 1 - 3) / 2)
            } else if self.layout.app.width < 120 {
                (visible_height / 2).saturating_sub((3 + 9 - 3) / 2)
            } else {
                (visible_height / 2).saturating_sub((3 + 11 - 3) / 2)
            };

        for idx in 0..dummies { list_items.push(ListItem::from(Line::default())); }

        if self.layout.app.width < 80 {
            list_items.push(ListItem::from(Line::from(Span::styled(format!("guita╭"), Style::default().fg(COLOR_GRASS))).centered()));
        } else if self.layout.app.width < 120 {
            list_items.push(ListItem::from(Line::from(Span::styled(format!("                    :#   :#                 "), Style::default().fg(COLOR_GRASS))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("                         L#                 "), Style::default().fg(COLOR_GRASS))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("  .##5#^.  .#   .#  :C  #C6#   #?##:        "), Style::default().fg(COLOR_GRASS))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("  #B   #G  C#   #B  #7   B?        G#       "), Style::default().fg(COLOR_GRASS))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("  #4   B5  B5   B5  B5   B5    1B5B#G  .a###"), Style::default().fg(COLOR_GREEN))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("  #b   5?  ?B   B5  B5   B5   ##   ##  B?   "), Style::default().fg(COLOR_GREEN))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("  .#B~6G!  .#6#~G.  #5   ~##  .##Y~#.  !#   "), Style::default().fg(COLOR_GREEN))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("      .##                              !B   "), Style::default().fg(COLOR_GREEN))).centered()));
            list_items.push(ListItem::from(Line::from(Span::styled(format!("     ~G#                               ~?   "), Style::default().fg(COLOR_GREEN))).centered()));
        } else {
            list_items.push(ListItem::from(Line::from(Span::styled(format!("                                 :GG~        .?Y.                                "), Style::default().fg(COLOR_GRASS))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("       ....        ..      ..   .....      . ^BG: ..       .....                 "), Style::default().fg(COLOR_GRASS))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("    .7555YY7JP^   ~PJ     ~PJ  ?YY5PP~    7YY5BGYYYYJ.   J555YY557.              "), Style::default().fg(COLOR_GRASS))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("   .5B?.  :JBB~   !#5     !#5  ...PB~     ...^BG:....    ~:.   .7#5           :^^"), Style::default().fg(COLOR_GRASS))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("   7#5     .GB~   !B5     !B5     PB~        :BG.        .~7??J?JBG:      .~JPPPY"), Style::default().fg(COLOR_GRASS))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("   ?#Y      PB~   !B5     !B5     PB~        :BG.       7GP7~^^^!BG:     ~5GY!:. "), Style::default().fg(COLOR_GREEN))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("   ^GB~    7BB~   ^BG.   .YB5     5#7        :BB:       P#!     JBG:    ^GG7     "), Style::default().fg(COLOR_GREEN))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("    ^5G5JJYJPB~    JBP???YYB5     ^5GYJJ?.    7GPJ???.  ~PGJ77?5J5B!    JG5      "), Style::default().fg(COLOR_GREEN))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("      .^~^..GB:     :~!!~. ^^       :~~~~      .^~~~~    .^!!!~. .^:    JG5      "), Style::default().fg(COLOR_GREEN))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("    .?!^^^!5G7                                                          YB5      "), Style::default().fg(COLOR_GREEN))).centered()));    
            list_items.push(ListItem::from(Line::from(Span::styled(format!("    .!?JJJ?!:                                                           75?      "), Style::default().fg(COLOR_GREEN))).centered()));    
        }
        list_items.push(ListItem::from(Line::default()));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(format!("made with ♡"), Style::default().fg(COLOR_TEXT))
        ]).centered()));
        list_items.push(ListItem::from(Line::from(vec![
            Span::styled(format!("https://github.com/asinglebit/guitar"), Style::default().fg(COLOR_TEXT))
        ]).centered()));

        // Setup the list
        let list = List::new(list_items)
            .block(
                Block::default()
                    .padding(padding)
            );

        // Render the list
        frame.render_widget(list, self.layout.app);
    }
}
