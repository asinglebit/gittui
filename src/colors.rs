use std::collections::HashMap;
use git2::{Oid};
use ratatui::style::Color;

pub struct Colors {
    color_map: HashMap<String, Color>,
    id_map: HashMap<String, usize>,
    next_color: usize,
    next_id: usize,
    palette: Vec<Color>,
}

impl Colors {
    pub fn new() -> Self {
        Self {
            color_map: HashMap::new(),
            id_map: HashMap::new(),
            next_color: 0,
            next_id: 0,
            palette: vec![
                Color::Cyan,
                Color::Magenta,
                Color::Red,
                Color::Green,
                Color::Yellow,
                Color::Blue,
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

    pub fn get_color(&self, branch: &String) -> Color {
        *self.color_map.get(branch).unwrap_or(&Color::White)
    }
}
