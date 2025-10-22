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
        Direction
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
            fill_width
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
        self.settings_selections = Vec::new();

        lines.push(Line::default());
        if self.layout.app.width < 120 && self.layout.app.height > 24 {
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::from(Span::styled("                    :#   :#                 ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled("                         L#                 ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled("  .##5#^.  .#   .#  :C  #C6#   #?##:        ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled("  #B   #G  C#   #B  #7   B?        G#       ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());
            lines.push(Line::from(Span::styled("  #4   B5  B5   B5  B5   B5    1B5B#G  .a###".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled("  #b   5?  ?B   B5  B5   B5   ##   ##  B?   ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled("  .#B~6G!  .#6#~G.  #5   ~##  .##Y~#.  !#   ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled("      .##                              !B   ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());
            lines.push(Line::from(Span::styled("     ~G#                               ~?   ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
        } else if self.layout.app.height > 30{
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::from(Span::styled("                                 :GG~        .?Y.                                ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled("       ....        ..      ..   .....      . ^BG: ..       .....                 ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled("    .7555YY7JP^   ~PJ     ~PJ  ?YY5PP~    7YY5BGYYYYJ.   J555YY557.              ".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled("   .5B?.  :JBB~   !#5     !#5  ...PB~     ...^BG:....    ~:.   .7#5           :^^".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled("   7#5     .GB~   !B5     !B5     PB~        :BG.        .~7??J?JBG:      .~JPPPY".to_string(), Style::default().fg(self.theme.COLOR_GRASS))).centered());    
            lines.push(Line::from(Span::styled("   ?#Y      PB~   !B5     !B5     PB~        :BG.       7GP7~^^^!BG:     ~5GY!:. ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled("   ^GB~    7BB~   ^BG.   .YB5     5#7        :BB:       P#!     JBG:    ^GG7     ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled("    ^5G5JJYJPB~    JBP???YYB5     ^5GYJJ?.    7GPJ???.  ~PGJ77?5J5B!    JG5      ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled("      .^~^..GB:     :~!!~. ^^       :~~~~      .^~~~~    .^!!!~. .^:    JG5      ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled("    .?!^^^!5G7                                                          YB5      ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::from(Span::styled("    .!?JJJ?!:                                                           75?      ".to_string(), Style::default().fg(self.theme.COLOR_GREEN))).centered());    
            lines.push(Line::default());
            lines.push(Line::default());
            lines.push(Line::default());
        }

        lines.push(Line::from(vec![
            Span::styled(fill_width("credentials", "", max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT))
        ]).centered());
        lines.push(Line::default());

        lines.push(Line::from(Span::styled(fill_width("name:", name.unwrap().as_str(), max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT).bg(self.theme.COLOR_GREY_900))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::from(Span::styled(fill_width("email:", email.unwrap().as_str(), max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::from(Span::styled(fill_width("authorization:", "external ssh agent", max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT).bg(self.theme.COLOR_GREY_900))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::default());
        lines.push(Line::from(Span::styled(fill_width("themes:", "", max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT))).centered());
        lines.push(Line::default());

        lines.push(Line::from(Span::styled(fill_width("classic", format!("({})", if self.theme.name == ThemeNames::Classic {"*"} else {" "}).as_str(), max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT).bg(self.theme.COLOR_GREY_900))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::from(Span::styled(fill_width("ansi", format!("({})", if self.theme.name == ThemeNames::Ansi {"*"} else {" "}).as_str(), max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::from(Span::styled(fill_width("monochrome", format!("({})", if self.theme.name == ThemeNames::Monochrome {"*"} else {" "}).as_str(), max_text_width / 2), Style::default().fg(self.theme.COLOR_TEXT).bg(self.theme.COLOR_GREY_900))).centered());
        
        // Record the line index as selectable
        self.settings_selections.push(lines.len() - 1);

        lines.push(Line::default());
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(" ╭─────────────────────────────────────────────────────────────────────╮".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [_]   [_][_][_][_] [_][_][_][_] [_][_][_][_] [_][_][_] [_][_][_][_] │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │                                                                     │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [`][1][2][_][_][_][_][_][_][_][_][_][_][___] [_][◼][◼] [_][_][_][_] │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [__][_][_][_][r][_][_][u][_][o][p][_][_][◼ │ [_][◼][◼] [_][_][_][ | │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [___][a][s][_][f][_][h][j][_][_][_][_][_][_│           [_][_][_][_| │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [◼][_][_][_][c][_][_][_][m][_][.][_][______]    [◼]    [_][_][_][ | │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" │ [◼_][_][__][_____________________][__][_][_] [_][◼][_] [____][.][_| │".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::from(Span::styled(" ╰─────────────────────────────────────────────────────────────────────╯".to_string(), Style::default().fg(self.theme.COLOR_GREY_800))).centered());    
        lines.push(Line::default());
        lines.push(Line::default());

        render_keybindings(&self.theme, &self.keymap, max_text_width / 2).iter().enumerate().for_each(|(idx, kb_line)| {
            let spans: Vec<Span> = kb_line.clone().spans.iter().map(|span| {
                let mut style = span.style;
                if idx % 2 == 0 { style = style.bg(self.theme.COLOR_GREY_900); }
                Span::styled(span.content.clone(), style)
            }).collect();
            lines.push(Line::from(spans).centered());
            
            // Record the line index as selectable
            self.settings_selections.push(lines.len() - 1);
        });

        // Get vertical dimensions
        let total_lines = lines.len();
        let visible_height = self.layout.graph.height as usize;

        // Snap to nearest selectable line if needed
        if !self.settings_selections.contains(&self.settings_selected) {
            // Find nearest selectable line above or below
            let mut nearest = None;

            // Moving down
            if self.last_input_direction == Some(Direction::Down) {
                nearest = self.settings_selections.iter()
                    .copied()
                    .find(|&i| i > self.settings_selected);
            }

            // Moving up
            if nearest.is_none() && self.last_input_direction == Some(Direction::Up) {
                nearest = self.settings_selections.iter()
                    .rev()
                    .copied()
                    .find(|&i| i < self.settings_selected);
            }

            // Fallback to nearest by distance if neither direction flag is set
            if nearest.is_none() {
                nearest = self.settings_selections
                    .iter()
                    .min_by_key(|&&i| i.abs_diff(self.settings_selected))
                    .copied();
            }

            if let Some(target) = nearest {
                self.settings_selected = target;
            }
        }
        
        // Calculate sticky scroll
        let start = (self.settings_selected + 1).saturating_sub(visible_height);
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
                        style = style.bg(self.theme.COLOR_GREY_800);
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
                self.theme.COLOR_GREY_600
            } else {
                self.theme.COLOR_BORDER
            }));

        // Render the scrollbar
        frame.render_stateful_widget(scrollbar, self.layout.app, &mut scrollbar_state);
    }
}
