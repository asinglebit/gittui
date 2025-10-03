use std::{collections::HashMap};
use git2::{BranchType, Commit, Oid, Repository, Time};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::colors::Colors;

#[derive(Clone)]
struct CommitMetadata {
    sha: Oid,
    parents: Vec<Oid>
}

pub fn get_commits(repo: &Repository) -> (Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>) {
    
    let mut colors = Colors::new();
    let mut graph = Vec::new();
    let mut branches = Vec::new();
    let mut messages = Vec::new();
    let mut buffer = Vec::new();
    
    let mut _buffer: Vec<CommitMetadata> = Vec::new();        
    let _tips: HashMap<Oid, Vec<String>> = get_tips(&repo);
    let _branches: HashMap<Oid, Vec<String>> = get_branches(&repo, &_tips);
    let _timestamps: HashMap<Oid, Time> = get_timestamps(&repo, &_branches);
    let _sorted: Vec<Oid> = sort(&_timestamps);
    
    for sha in _sorted {
        let commit = repo.find_commit(sha).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let metadata = CommitMetadata {
            sha,
            parents,
        };
        
        let mut spans_graph = Vec::new();
        
        // Update
        update_buffer(&mut _buffer, metadata);

        // Flags
        let mut is_commit_found = false;

        // Symbols
        let symbol_commit = if _tips.contains_key(&sha) { "● " } else { "○ " };
        let symbol_pass = "│ ";
        let symbol_empty = "  ";

        // Preassembled line
        let mut preassembled: HashMap<i32, Vec<(String, Color)>> = HashMap::new();
        let mut preassemble = |layer: i32, symbol: String, sha: &Oid| {
            preassembled.entry(layer).or_default().push((symbol, colors.get_branch_color(&sha, &_branches)));
        };

        // Find double shas if present
        let mut double: Option<CommitMetadata> = None;
        let mut last: Option<Oid> = None;
        for metadata in &_buffer {
            if let Some(last_oid) = last {
                if last_oid == metadata.sha {
                    double = Some(CommitMetadata {
                        sha: metadata.sha,
                        parents: vec![*metadata.parents.first().unwrap()],
                    });
                }
            }
            last = Some(metadata.sha);
        }

        if let Some(double) = double {
            // If succeded, remove the double from the buffer
            _buffer.retain(|metadata|
                metadata.sha != double.sha || *metadata.parents.first().unwrap() != *double.parents.first().unwrap()
            );
            
            // And add a merge layer, with connected merge arrow
            let double_sha = double.sha;
            let double_parent = *double.parents.first().unwrap(); // safe because we constructed it with a parent

            let mut in_merge_path = false; // tracks if we are currently along the merge path
            let mut first_found = false;

            // And add a merge layer, with connected merge arrow
            for metadata in &_buffer {
                let current_sha = metadata.sha;
                let current_parent_sha = *metadata.parents.first().unwrap();

                if current_sha == double_sha || current_parent_sha == double_parent {
                    if !first_found {
                        first_found = true;
                        if current_sha == double_sha {
                            preassemble(1, "  ".to_string(), &current_sha);
                            preassemble(1, "≺─".to_string(), &current_sha);
                        } else {
                            // preassemble(1, "≺─".to_string(), &metadata.sha);
                        }
                    } else {
                        if current_sha == double_sha {
                            // preassemble(1, "  ≺─".to_string(), &current_sha);
                        } else {
                            preassemble(1, "⎨ ".to_string(), &current_sha);
                        }
                    }
                    in_merge_path = true;
                } else if in_merge_path {
                    in_merge_path = false;
                }

                if in_merge_path && current_sha != double_sha && current_parent_sha != double_parent {
                    preassemble(1, "──".to_string(), &current_sha);
                }
            }
        }

        // Add commit layer
        for metadata in &_buffer {
            if metadata.parents.len() == 0 { continue }
            let is_at_parent = sha == metadata.sha;
            
            if !is_commit_found {
                if is_at_parent {
                    is_commit_found = true;
                    preassemble(0, symbol_commit.to_string(), &metadata.sha);
                    preassemble(2, symbol_empty.to_string(), &metadata.sha);
                } else {
                    preassemble(0, symbol_empty.to_string(), &metadata.sha);
                    preassemble(2, symbol_pass.to_string(), &metadata.sha);
                }
            } else {
                preassemble(0, symbol_empty.to_string(), &metadata.sha);
                preassemble(2, symbol_pass.to_string(), &metadata.sha);
            };
            
        }        
        if !is_commit_found {
            preassemble(0, symbol_commit.to_string(), &sha);
            preassemble(2, symbol_empty.to_string(), &sha);
        }

        // Add filler layer

        // Merge preassembled layers
        // Determine max length of layers
        let max_len = preassembled.values().map(|v| v.len()).max().unwrap_or(0);
        for i in 0..max_len {
            let mut symbol = "  "; // default whitespace
            let mut color: Color = Color::Black;
            for layer in 0..3 as i32 {
                if let Some(vec) = preassembled.get(&layer) {
                    if i < vec.len() {
                        let (s, _color) = &vec[i];
                        if s.trim() != "" {
                            symbol = s; // take the first non-whitespace symbol
                            color = *_color;
                            break;
                        }
                    }
                }
            }
            spans_graph.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
        }

        // Render
        // for layer in preassembled.keys() {
        //     if let Some(items) = preassembled.get(&layer) {
        //         for (symbol, color) in items {
        //             spans_graph.push(Span::styled(symbol.to_string(), Style::default().fg(*color)));
        //         }
        //     }
        // }

        // Serialize        
        serialize_graph(&sha, &mut graph, spans_graph);
        serialize_branches(&sha, &mut branches, &_tips, &_branches, &colors);
        serialize_messages(&commit, &mut messages);
        serialize_buffer(&sha, &_buffer, &_timestamps, &mut buffer);
    }

    (graph, branches, messages, buffer)
}

fn update_buffer(buffer: &mut Vec<CommitMetadata>, metadata: CommitMetadata) {
    let sha = metadata.sha;
    if let Some(first_idx) = buffer.iter().position(|inner| inner.parents.contains(&sha)) {
        buffer[first_idx] = metadata;
        let keep_ptr = buffer[first_idx].parents.as_ptr();
        buffer.retain(|inner| !inner.parents.contains(&sha) || inner.parents.as_ptr() == keep_ptr);
    } else {
        buffer.push(metadata);
    }
    flatten(buffer);
}

fn flatten(buffer: &mut Vec<CommitMetadata>) {
    let mut i = 0;
    while i < buffer.len() {
        if buffer[i].parents.len() > 1 {
            let mut inner = buffer.remove(i);
            for (j, item) in inner.parents.drain(..).enumerate() {
                let metadata = CommitMetadata {
                    sha: inner.sha,
                    parents: vec![item]
                };
                buffer.insert(i + j, metadata);
            }
            i += inner.parents.len();
        } else {
            i += 1;
        }
    }
}

fn sort(_timestamps: &HashMap<Oid, Time>) -> Vec<Oid> {
    let mut shas: Vec<Oid> = _timestamps.keys().copied().collect();
    shas.sort_by(|a, b| {
        let ta = _timestamps[a].seconds();
        let tb = _timestamps[b].seconds();
        tb.cmp(&ta).then(a.cmp(b))
    });
    shas
}

fn get_tips(repo: &Repository) -> HashMap<Oid, Vec<String>> {
    let mut tips: HashMap<Oid, Vec<String>> = HashMap::new();
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            if let Some(target) = branch.get().target() {
                let name = branch.name().unwrap().unwrap_or("unknown").to_string();
                tips.entry(target).or_default().push(name);
            }
        }
    }
    tips
}

fn get_branches(repo: &Repository, tips: &HashMap<Oid, Vec<String>>) -> HashMap<Oid, Vec<String>> {
    let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
    for (sha_tip, names) in tips {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(*sha_tip).unwrap();
        for sha_step in revwalk {
            let sha = sha_step.unwrap();
            for name in names {
                map.entry(sha).or_default().push(name.clone());
            }
        }
    }
    map
}

fn get_timestamps(repo: &Repository, _branches: &HashMap<Oid, Vec<String>>) -> HashMap<Oid, Time> {
    _branches
        .keys()
        .map(|&sha| {
            let commit = repo.find_commit(sha).unwrap();
            let time = commit.author().when();
            (sha, time)
        })
        .collect()
}

fn serialize_graph(sha: &Oid, graph: &mut Vec<Line<>>, spans_graph: Vec<Span<'static>>) {
    let span_sha = Span::styled(sha.to_string()[..2].to_string(), Style::default().fg(Color::DarkGray));
    let mut spans = Vec::new();
    spans.push(span_sha);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

fn serialize_branches(sha: &Oid, branches: &mut Vec<Line<>>, _tips: &HashMap<Oid, Vec<String>>, _branches: &HashMap<Oid, Vec<String>>, colors: &Colors) {
    let mut spans = Vec::new();
    let span_tips: Vec<Span<'_>> = _tips.get(&sha).map(|branches| {
        branches.iter().map(|branch| {
            Span::styled(format!("* {} ", branch), Style::default().fg(colors.get_color(branch)))
        }).collect()
    }).unwrap_or_default();
    spans.extend(span_tips);
    // let span_branches = Span::styled(_branches.get(&sha).unwrap().join(","), Style::default().fg(Color::Yellow));
    // spans.push(span_branches);
    branches.push(Line::from(spans));
}

fn serialize_messages(commit: &Commit<'_>, messages: &mut Vec<Line<>>) {
    let mut spans = Vec::new();
    let span_message = Span::styled(commit.summary().unwrap_or("<no message>").to_string(), Style::default().fg(Color::DarkGray));
    spans.push(span_message);
    messages.push(Line::from(spans));
}

fn serialize_buffer(_sha: &Oid, _buffer: &Vec<CommitMetadata>, _timestamps: &HashMap<Oid, Time>, buffer: &mut Vec<Line<>>) {
    let mut spans = Vec::new();

    // let time = _timestamps.get(_sha).unwrap().seconds();
    // let offset = _timestamps.get(_sha).unwrap().offset_minutes();
    // let span_timestamp = Span::styled(format!("{}:{:<3} ", time, offset), Style::default().fg(Color::DarkGray));
    // spans.push(span_timestamp);
    
    let formatted_buffer: String = _buffer.iter().map(|metadata| {
            format!(
                "{:.2}({})",
                metadata.sha,
                metadata.parents.iter().map(|oid| {format!("{:.2}", oid)}).collect::<Vec<String>>().join(" - ")
            )
        }).collect::<Vec<String>>().join(" ");
    let span_buffer = Span::styled(formatted_buffer, Style::default().fg(Color::DarkGray));
    spans.push(span_buffer);

    buffer.push(Line::from(spans));
}
