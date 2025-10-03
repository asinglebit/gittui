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
    tip: Option<Oid>,
    parents: Vec<Oid>
}
#[derive(Eq, Hash, PartialEq)]
enum Layers {
    Commits = 0,
    Merges = 1,
    Pipes = 2
}

pub fn get_commits(repo: &Repository) -> (Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>) {
    
    let mut colors = Colors::new();
    let mut graph = Vec::new();
    let mut branches = Vec::new();
    let mut messages = Vec::new();
    let mut buffer = Vec::new();
    
    let mut _offset_prev: HashMap<Oid, i32> = HashMap::new();
    let mut _offset: HashMap<Oid, i32> = HashMap::new();
    let mut _buffer_prev: Vec<CommitMetadata> = Vec::new();     
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
            tip: if _tips.contains_key(&sha) {Some(sha)} else {None},
            parents,
        };
        
        let mut spans_graph = Vec::new();
        
        // Update
        update_buffer(&mut _buffer, &mut _offset, metadata);

        // Symbols
        let symbol_commit = if _tips.contains_key(&sha) { "● " } else { "○ " };
        let symbol_vertical = "│ ";
        let symbol_horizontal = "──";
        let symbol_empty = "  ";
        let symbol_merge_left_to = "⏴─";
        let symbol_merge_left_from = "┤ ";
        let symbol_branch_right = "╯ ";

        // Layers
        let mut layers: HashMap<Layers, Vec<(String, Color)>> = HashMap::new();
        let mut layer = |layer: Layers, symbol: String, sha: &Oid| {
            layers.entry(layer).or_default().push((symbol, colors.get_branch_color(&sha, &_branches)));
        };

        // Merge layer
        {
            let mut merge: Option<CommitMetadata> = None;

            // Find merge if present
            {
                let mut last: Option<Oid> = None;
                for metadata in &_buffer {
                    if let Some(last_oid) = last {
                        if last_oid == metadata.sha {
                            merge = Some(CommitMetadata {
                                sha: metadata.sha,
                                tip: None,
                                parents: vec![*metadata.parents.first().unwrap()],
                            });
                        }
                    }
                    last = Some(metadata.sha);
                }
            }
            
            // If merge is present, compute the merge layer
            if let Some(mut merge) = merge {

                // If succeded, remove the merge from the buffer
                _buffer.retain(|metadata|
                    metadata.sha != merge.sha || *metadata.parents.first().unwrap() != *merge.parents.first().unwrap()
                );
                
                // Now find the correct tip
                for metadata in &_buffer {
                    if merge.parents.first().unwrap() == metadata.parents.first().unwrap() {
                        merge.tip = Some(metadata.tip.unwrap_or(Oid::zero()));
                    }
                }
                
                let merge_sha = merge.sha;
                let merge_tip = merge.tip.unwrap_or(Oid::zero());
                let merge_parent = *merge.parents.first().unwrap_or(&Oid::zero()); // Safe because we constructed it with a parent
                let mut in_merge_path = false; // Tracks if we are currently along the merge path
                let mut first_found = false; // Used to track if we found commit itself or the parent first

                // And add a merge layer, with connected merge arrow
                for metadata in &_buffer {
                    let current_sha = metadata.sha;
                    let current_parent_sha = *metadata.parents.first().unwrap();
                    if current_sha == merge_sha || current_parent_sha == merge_parent {
                        if !first_found {
                            first_found = true;
                            if current_sha == merge_sha {
                                layer(Layers::Merges, symbol_empty.to_string(), &merge_tip);
                                layer(Layers::Merges, symbol_merge_left_to.to_string(), &merge_tip);
                            } else {
                                // layer(Layers::Merges, "≺─".to_string(), &metadata.sha);
                            }
                        } else {
                            if current_sha == merge_sha {
                                // layer(Layers::Merges, "  ≺─".to_string(), &current_sha);
                            } else {
                                layer(Layers::Merges, symbol_merge_left_from.to_string(), &merge_tip);
                            }
                        }
                        in_merge_path = true;
                    } else if in_merge_path {
                        in_merge_path = false;
                    }

                    if in_merge_path && current_sha != merge_sha && current_parent_sha != merge_parent {
                        layer(Layers::Merges, symbol_horizontal.to_string(), &merge_tip);
                    }
                }
            }
        }

        // Commit and Pipe layers
        {
            if _buffer_prev.len() > _buffer.len() {
                // If buffer decreased (meaning theres a branch)
                let mut is_commit_found = false;
                for metadata in &_buffer_prev {
                    if metadata.parents.len() == 0 { continue }
                    let metadata_parent = *metadata.parents.first().unwrap_or(&Oid::zero());
                    if !is_commit_found {
                        if sha == metadata_parent {
                            is_commit_found = true;
                            layer(Layers::Commits, symbol_commit.to_string(), &metadata.sha);
                            layer(Layers::Pipes, symbol_empty.to_string(), &metadata.sha);
                        } else {
                            layer(Layers::Commits, symbol_empty.to_string(), &metadata.sha);
                            layer(Layers::Pipes, symbol_vertical.to_string(), &metadata.sha);
                        }
                    } else {
                        if sha == metadata_parent {
                            layer(Layers::Commits, symbol_empty.to_string(), &metadata.tip.unwrap_or(Oid::zero()));
                            layer(Layers::Pipes, symbol_branch_right.to_string(), &metadata.tip.unwrap_or(Oid::zero()));
                        } else {
                            layer(Layers::Commits, symbol_empty.to_string(), &metadata.sha);
                            layer(Layers::Pipes, symbol_vertical.to_string(), &metadata.sha);
                        }
                    }                    
                }
            } else {
                // Otherwise (meaning we reached a tip, merge or a non-branching commit)
                let mut is_commit_found = false;
                for metadata in &_buffer {
                    if metadata.parents.len() == 0 { continue }
                    if sha == metadata.sha {
                        is_commit_found = true;
                        layer(Layers::Commits, symbol_commit.to_string(), &metadata.tip.unwrap_or(Oid::zero()));
                        layer(Layers::Pipes, symbol_empty.to_string(), &metadata.tip.unwrap_or(Oid::zero()));
                    } else {
                        layer(Layers::Commits, symbol_empty.to_string(), &metadata.sha);
                        layer(Layers::Pipes, symbol_vertical.to_string(), &metadata.sha);
                    }

                    if _offset_prev.contains_key(&metadata.tip.unwrap_or(Oid::zero())) {
                        let offset = _offset_prev.get(&metadata.tip.unwrap_or(Oid::zero())).unwrap().clone();
                        for i in 0..offset {
                            if i == offset - 1 {
                                layer(Layers::Commits, symbol_empty.to_string(), &metadata.sha);
                                layer(Layers::Pipes, symbol_branch_right.to_string(), &metadata.sha);
                            } else {
                                layer(Layers::Commits, symbol_empty.to_string(), &metadata.sha);
                                layer(Layers::Pipes, symbol_horizontal.to_string(), &metadata.sha);
                            }
                        }
                    }
                }        
                if !is_commit_found {
                    layer(Layers::Commits, symbol_commit.to_string(), &sha);
                    layer(Layers::Pipes, symbol_empty.to_string(), &sha);
                }
            }
        }

        // Assemble layers into the graph
        {
            // Determine max length of layers
            let max_len = layers.get(&Layers::Commits).unwrap().len();

            // For each token
            for token_index in 0..max_len {
                let mut symbol = symbol_empty;
                let mut color: Color = Color::Black;

                // For each layer
                for layer in [Layers::Commits, Layers::Merges, Layers::Pipes] {
                    if let Some(tokens) = layers.get(&layer) {
                        if token_index < tokens.len() {

                            // Take the first non-whitespace symbol
                            let (_symbol, _color) = &tokens[token_index];
                            if _symbol.trim() != "" {
                                symbol = _symbol;
                                color = *_color;
                                break;
                            }
                        }
                    }
                }
                spans_graph.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
            }
        }

        _buffer_prev = _buffer.clone();
        _offset_prev = _offset.clone();

        // Serialize        
        serialize_graph(&sha, &mut graph, spans_graph);
        serialize_branches(&sha, &mut branches, &_tips, &_branches, &colors);
        serialize_messages(&commit, &mut messages);
        serialize_buffer(&sha, &_buffer, &mut _offset, &_timestamps, &mut buffer);
    }

    (graph, branches, messages, buffer)
}

fn update_buffer(buffer: &mut Vec<CommitMetadata>, offset: &mut HashMap<Oid, i32>, metadata: CommitMetadata) {
    
    // Clear offsets from the previous iteration
    offset.clear();

    // Replace or append buffer metadata    
    if let Some(first_idx) = buffer.iter().position(|inner| inner.parents.contains(&metadata.sha)) {
        let sha = metadata.sha;
        let tip = buffer[first_idx].tip;
        buffer[first_idx] = metadata;
        buffer[first_idx].tip = tip;
        let keep_ptr = buffer[first_idx].parents.as_ptr();

        // Calculate and store offsets for the next iteration
        let old_len = buffer.len() as i32;
        buffer.retain(|inner| !inner.parents.contains(&sha) || inner.parents.as_ptr() == keep_ptr);
        let new_len = buffer.len() as i32;
        let _offset = old_len - new_len;
        
        if _offset > 0 {
            let mut found = false;
            for item in buffer.iter() {
                let _tip = item.tip.unwrap_or(Oid::zero());
                if !found {
                    if tip.unwrap_or(Oid::zero()) == _tip {
                        found = true;
                    }
                } else {
                    offset.entry(_tip).or_insert(_offset);
                }
            }
        }
    } else {
        buffer.push(metadata);
    }

    // Flatten merge
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
                    tip: inner.tip,
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
            Span::styled(format!("● {} ", branch), Style::default().fg(colors.get_color(branch)))
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

fn serialize_buffer(_sha: &Oid, _buffer: &Vec<CommitMetadata>, _offset: &mut HashMap<Oid, i32>, _timestamps: &HashMap<Oid, Time>, buffer: &mut Vec<Line<>>) {
    let mut spans = Vec::new();

    // let time = _timestamps.get(_sha).unwrap().seconds();
    // let offset = _timestamps.get(_sha).unwrap().offset_minutes();
    // let span_timestamp = Span::styled(format!("{}:{:<3} ", time, offset), Style::default().fg(Color::DarkGray));
    // spans.push(span_timestamp);
    
    let formatted_buffer: String = _buffer.iter().map(|metadata| {
            format!(
                "{:.2}({})[{:.2}]",
                metadata.sha,
                metadata.parents.iter().map(|oid| {format!("{:.2}", oid)}).collect::<Vec<String>>().join(" - "),
                metadata.tip.unwrap_or(Oid::zero()),
            )
        }).collect::<Vec<String>>().join(" ");
    let formatted_offsets: String = _offset.iter().map(|(oid, offset)| {
            format!(
                "{:.2}|{}",
                oid,
                offset,
            )
        }).collect::<Vec<String>>().join(" ");
    let span_buffer = Span::styled(format!("{:<50} : {}", formatted_buffer, formatted_offsets), Style::default().fg(Color::DarkGray));
    spans.push(span_buffer);

    buffer.push(Line::from(spans));
}
