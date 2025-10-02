use std::env;
use std::path::PathBuf;
use std::collections::{HashMap};
use git2::{BranchType, Oid, Repository};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::branch_manager::BranchManager;

pub fn get_commits() -> (Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>) {
    
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };
    let repo = Repository::open(path).expect("Could not open repo");

    // Collect branch tips
    let branch_commit_tuples = collect_branch_tips(&repo);

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
    let mut messages = Vec::new();

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
        let mut branch_spans: Vec<Span<'_>> = Vec::new();
        if let Some(branch_prints) = branch_tips.get(&oid) {
            for branch in branch_prints {
                // Create a Span for each branch name
                let span = Span::styled(
                    format!("* {} ", branch),
                    Style::default().fg(branch_colors.get_color(&branch))
                );
                branch_spans.push(span);
            }
        }
        
        // Commit message
        let commit_msg = commit.summary().unwrap_or("<no message>").to_string();

        // Short SHA
        let sha_span = Span::styled(oid.to_string()[..8].to_string(), Style::default().fg(Color::DarkGray));

        // Whole branches
        let whole_branch_spans = Span::styled(format!("{:<30}", map_branch_commits.get(&oid).unwrap().join(",")), Style::default().fg(Color::Yellow));

        // Commit message
        let msg_span = Span::styled(format!("{:<10}", commit_msg), Style::default().fg(Color::DarkGray));

        // Combine into a Line
        let mut structure_spans = Vec::new();
        structure_spans.push(sha_span);
        structure_spans.push(Span::raw(" ".to_string()));
        structure_spans.extend(tree_spans);
        structure.push(Line::from(structure_spans));

        let mut descriptors_spans = Vec::new();
        descriptors_spans.extend(branch_spans);
        // descriptors_spans.push(whole_branch_spans);
        descriptors.push(Line::from(descriptors_spans));

        let mut messages_spans = Vec::new();
        messages_spans.push(msg_span);
        messages.push(Line::from(messages_spans));

        // Update buffer for tree hierarchy
        split_inner(&mut buffer);
        replace_or_append_oid(&mut buffer, oid, parent_oids);
    }

    (structure, descriptors, messages)
}

fn collect_branch_tips(repo: &Repository) -> Vec<(String, Oid)> {
    let mut branch_commit_tuples = Vec::new();
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            if let Some(target) = branch.get().target() {
                let name = branch.name().unwrap().unwrap_or("unknown").to_string();
                branch_commit_tuples.push((name, target));
            }
        }
    }
    branch_commit_tuples
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