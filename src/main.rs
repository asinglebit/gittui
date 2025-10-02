use std::collections::{HashMap};
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use git2::{BranchType, Oid, Repository};
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

// fn main() {
//     let lines = get_commits();
//     for line in lines {
//         println!("{line}");
//     }
// }

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (structure, descriptors) = get_commits();
        let structure_text = ratatui::text::Text::from(structure);
        let descriptors_text = ratatui::text::Text::from(descriptors);

        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([ratatui::layout::Constraint::Percentage(20), ratatui::layout::Constraint::Percentage(80)])
            .split(area);

        ratatui::widgets::Paragraph::new(structure_text.clone())
            .left_aligned()
            .block(ratatui::widgets::Block::bordered())
            .render(chunks[0], buf);

        ratatui::widgets::Paragraph::new(descriptors_text)
            .left_aligned()
            .block(ratatui::widgets::Block::bordered())
            .render(chunks[1], buf);
    }
}
pub struct BranchManager {
    color_map: HashMap<String, Color>,
    id_map: HashMap<String, usize>,
    next_color: usize,
    next_id: usize,
    palette: Vec<Color>,
}

impl BranchManager {
    pub fn new() -> Self {
        Self {
            color_map: HashMap::new(),
            id_map: HashMap::new(),
            next_color: 0,
            next_id: 0,
            palette: vec![
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
                Color::Magenta,
                Color::Cyan,
                Color::White,
                Color::LightRed,
                Color::LightGreen,
                Color::LightYellow,
                Color::LightBlue,
                Color::LightMagenta,
                Color::LightCyan,
            ],
        }
    }

    /// Given an Oid and a commit->branches map, returns the color of the **oldest branch**
    pub fn get_branch_color(&mut self, oid: &Oid, map_branch_commits: &HashMap<Oid, Vec<String>>) -> Color {
        // Get the branch list for this commit
        let branches = match map_branch_commits.get(oid) {
            Some(b) if !b.is_empty() => b.clone(),
            _ => vec!["default".to_string()],
        };

        // Assign IDs & colors to any new branches
        for branch in &branches {
            if !self.id_map.contains_key(branch) {
                self.id_map.insert(branch.clone(), self.next_id);
                self.next_id += 1;
            }
            if !self.color_map.contains_key(branch) {
                let color = self.palette[self.next_color % self.palette.len()];
                self.color_map.insert(branch.clone(), color);
                self.next_color += 1;
            }
        }

        // Sort branches by ID (older branches first)
        let mut sorted = branches.clone();
        sorted.sort_by_key(|b| self.id_map[b]);

        // Pick the oldest branch to determine the color
        let oldest = &sorted[0];
        *self.color_map.get(oldest).unwrap_or(&Color::White)
    }

    /// Get numeric ID of a branch
    pub fn get_branch_id(&self, branch: &str) -> Option<usize> {
        self.id_map.get(branch).copied()
    }
}


pub fn get_commits() -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let repo = Repository::open("../playground").expect("Could not open repo");

    // Collect branch tips
    let mut branch_commit_tuples = Vec::new();
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            let name = branch.name().unwrap().unwrap_or("unknown").to_string();
            if let Some(target) = branch.get().target() {
                branch_commit_tuples.push((name, target));
            }
        }
    }

    let mut branch_tips: HashMap<Oid, Vec<String>> = HashMap::new();
    for (branch, oid) in &branch_commit_tuples {
        branch_tips.entry(*oid).or_default().push(branch.clone());
    }

    // Map commit Oids to branches
    let map_branch_commits = map_commits_to_branches(&repo, &branch_commit_tuples);

    // Collect commit times for sorting
    let commit_times = map_commit_times(&repo, &map_branch_commits);

    // Sort commits by time (most recent first)
    let mut oids: Vec<_> = map_branch_commits.keys().copied().collect();
    oids.sort_by_key(|oid| commit_times[oid]);
    oids.reverse();

    let mut branch_colors = BranchManager::new();

    let mut buffer: Vec<Vec<Oid>> = Vec::new();
        
    let mut structure = Vec::new();
    let mut descriptors = Vec::new();

    for oid in oids {
        let commit = repo.find_commit(oid).unwrap();
        let parent_oids: Vec<Oid> = commit.parent_ids().collect();

        // Build tree markers as Spans
        let mut tree_spans = Vec::new();
        let mut found = false;

        if buffer.is_empty() {
            let symbol = if branch_tips.contains_key(&oid) { "*" } else { "·" };
            tree_spans.push(Span::styled(symbol.to_string(), Style::default().fg(branch_colors.get_branch_color(&oid, &map_branch_commits))));
        } else {
            for oid_tuple in &buffer {
                match oid_tuple.len() {
                    1 => {
                        let symbol = if oid == oid_tuple[0] {
                            if found { "┘ " } 
                            else { found = true; if branch_tips.contains_key(&oid) { "* " } else { "· " } }
                        } else { "│ " };
                        // tree_spans.push(Span::raw(symbol.to_string()));
                        tree_spans.push(Span::styled(symbol.to_string(), Style::default().fg(branch_colors.get_branch_color(&oid_tuple[0], &map_branch_commits))));
                    }
                    _ => {
                        let len = oid_tuple.len();
                        for (i, item) in oid_tuple.iter().enumerate() {
                            let symbol = match i {
                                0 => "├─",
                                x if x == len - 1 => {
                                    if oid == *item { found = true; if branch_tips.contains_key(&oid) { "*" } else { "·" } } 
                                    else { "┐" }
                                }
                                _ => "─",
                            };
                            // tree_spans.push(Span::raw(symbol.to_string()));
                        tree_spans.push(Span::styled(symbol.to_string(), Style::default().fg(branch_colors.get_branch_color(&oid_tuple[0], &map_branch_commits))));
                        }
                    }
                }
            }
            if !found {
                let symbol = if branch_tips.contains_key(&oid) { "*" } else { "·" };
                // tree_spans.push(Span::raw(symbol.to_string()));
                tree_spans.push(Span::styled(symbol.to_string(), Style::default().fg(branch_colors.get_branch_color(&oid, &map_branch_commits))));
            }
        }
        tree_spans.push(Span::raw(format!("{:<10}", ' ')));

        // Branch names
        let branch_span = if let Some(branch_prints) = branch_tips.get(&oid) {
            format!("* {}", branch_prints.join(", * "))
        } else { format!("") };
        
        // Commit message
        let commit_msg = commit.summary().unwrap_or("<no message>").to_string();

        // Short SHA
        let sha_span = Span::styled(oid.to_string()[..8].to_string(), Style::default().fg(Color::DarkGray));

        // Branch tips
        let branch_spans = Span::styled(format!("{:<20}", branch_span.clone()), Style::default().fg(Color::Yellow));

        // Whole branches
        let whole_branch_spans = Span::styled(format!("{:<30}", map_branch_commits.get(&oid).unwrap().join(",")), Style::default().fg(Color::Yellow));

        // Commit message
        let msg_span = Span::styled(format!("{:<10}", commit_msg), Style::default().fg(Color::Cyan));

        // Combine into a Line
        let mut structure_spans = Vec::new();
        structure_spans.push(sha_span);
        structure_spans.push(Span::raw(" ".to_string()));
        structure_spans.extend(tree_spans);
        structure.push(Line::from(structure_spans));

        let mut descriptors_spans = Vec::new();
        descriptors_spans.push(branch_spans);
        descriptors_spans.push(whole_branch_spans);
        descriptors_spans.push(msg_span);
        descriptors.push(Line::from(descriptors_spans));

        // Update buffer for tree hierarchy
        split_inner(&mut buffer);
        replace_or_append_oid(&mut buffer, oid, parent_oids);
    }

    (structure, descriptors)
}

fn map_commits_to_branches(repo: &Repository, branch_commit_tuples: &[(String, Oid)]) -> HashMap<Oid, Vec<String>> {
    let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
    for (branch_name, tip_oid) in branch_commit_tuples {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(*tip_oid).unwrap();
        for oid_result in revwalk {
            let oid = oid_result.unwrap();
            map.entry(oid).or_default().push(branch_name.clone());
        }
    }
    map
}

fn map_commit_times(repo: &Repository, map_branch_commits: &HashMap<Oid, Vec<String>>) -> HashMap<Oid, i64> {
    map_branch_commits.keys().map(|&oid| (oid, repo.find_commit(oid).unwrap().time().seconds())).collect()
}


fn split_inner(data: &mut Vec<Vec<Oid>>) {
    let mut i = 0;
    while i < data.len() {
        if data[i].len() > 1 {
            let mut inner = data.remove(i);
            for (j, item) in inner.drain(..).enumerate() {
                data.insert(i + j, vec![item]);
            }
            i += inner.len();
        } else {
            i += 1;
        }
    }
}

fn replace_or_append_oid(data: &mut Vec<Vec<Oid>>, target: Oid, replacement: Vec<Oid>) {
    if let Some(first_idx) = data.iter().position(|inner| inner.contains(&target)) {
        data[first_idx] = replacement;
        let keep_ptr = data[first_idx].as_ptr();
        data.retain(|inner| !inner.contains(&target) || inner.as_ptr() == keep_ptr);
    } else {
        data.push(replacement);
    }
}