use std::{collections::HashMap};
use chrono::{
    DateTime,
    FixedOffset,
    TimeZone,
    Utc
};
use git2::{BranchType, Commit, Delta, Oid, Repository, Status, StatusOptions, Time};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};
use crate::colors::*;

#[derive(Clone)]
struct CommitMetadata {
    sha: Oid,
    parents: Vec<Oid>,
}

impl CommitMetadata {
    fn dummy() -> Self {
        CommitMetadata {
            sha: Oid::zero(),
            parents: Vec::new(),
        }
    }

    fn is_dummy(&self) -> bool {
        self.sha == Oid::zero() && self.parents.is_empty()
    }
}

#[derive(Eq, Hash, PartialEq)]
enum Layers {
    Commits = 0,
    Merges = 1,
    Pipes = 2
}

struct ColorPicker {
    lanes: HashMap<usize, bool>,
    palette_a: [Color; 8],
    palette_b: [Color; 8]
}

impl Default for ColorPicker {
    fn default() -> Self {
        ColorPicker {
            lanes: HashMap::new(),
            palette_a: [
                COLOR_RED,
                COLOR_PURPLE,
                COLOR_INDIGO,
                COLOR_CYAN,
                COLOR_GREEN,
                COLOR_LIME,
                COLOR_AMBER,
                COLOR_GRAPEFRUIT,
            ],
            palette_b: [
                COLOR_PINK,
                COLOR_DURPLE,
                COLOR_BLUE,
                COLOR_TEAL,
                COLOR_GRASS,
                COLOR_YELLOW,
                COLOR_ORANGE,
                COLOR_BROWN,
            ]
        }
    }
}

impl ColorPicker {
    fn alternate(&mut self, lane: usize) {
        self.lanes.entry(lane).and_modify(|value| *value = !*value).or_insert(false);  
    }

    fn get(&self, lane: usize) -> Color {
        if self.lanes.get(&lane).copied().unwrap_or(false) {
            self.palette_b[lane % self.palette_b.len()]
        } else {
            self.palette_a[lane % self.palette_a.len()]
        }
    }
}

pub fn get_commits(repo: &Repository) -> (Vec<Oid>, Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>, Vec<Line<'static>>) {
    
    let mut color = ColorPicker::default();
    let mut graph = Vec::new();
    let mut branches = Vec::new();
    let mut messages = Vec::new();
    let mut buffer = Vec::new();
    let mut shas = Vec::new();
    
    let mut _buffer_prev: Vec<CommitMetadata> = Vec::new();
    let mut _buffer: Vec<CommitMetadata> = Vec::new();
    let _tips: HashMap<Oid, Vec<String>> = get_tips(&repo);
    let mut _tip_colors: HashMap<Oid, Color> = HashMap::new();
    let _branches: HashMap<Oid, Vec<String>> = get_branches(&repo, &_tips);
    let _timestamps: HashMap<Oid, (Time, Time, Time)> = get_timestamps(&repo, &_branches);
    let mut _sorted: Vec<Oid> = get_sorted_commits(&repo);
    let mut _not_found_mergers: Vec<Oid> = Vec::new();

    // Make a fake commit for unstaged changes
    let uncommitted_changes = get_uncommitted_changes(repo);
    let head = repo.head().unwrap();
    let head_sha = head.target().unwrap(); // Oid (SHA1)
    if uncommitted_changes.len() > 0 {        
        shas.push(Oid::zero());
        branches.push(Line::from(Span::styled("Uncommitted changes (1)", Style::default().fg(COLOR_GREY_400))));
        buffer.push(Line::from(Span::styled("--", Style::default().fg(COLOR_GREY_400))));
        graph.push(Line::from(vec![
            Span::styled("🞄🞄🞄🞄🞄🞄🞄 ", Style::default().fg(COLOR_TEXT)),
            Span::styled("◌", Style::default().fg(COLOR_GREY_400))
        ]));
        // _buffer.push(value);
        let parents: Vec<Oid> = vec![head_sha];
        let metadata = CommitMetadata {
            sha: Oid::from_str("0000000000000000000000000000000000000001").unwrap(),
            parents,
        };
        
        // Update
        update_buffer(&mut _buffer, &mut _not_found_mergers, metadata);
    }

    // Go through the commits, inferring the graph
    for sha in _sorted {
        let commit = repo.find_commit(sha).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let metadata = CommitMetadata {
            sha,
            parents,
        };
        
        let mut spans_graph = Vec::new();
        
        // Update
        update_buffer(&mut _buffer, &mut _not_found_mergers, metadata);

        // Symbols
        let symbol_commit_branch = "●";
        let symbol_commit = "○";
        let symbol_vertical = "│";
        let symbol_vertical_dotted = "⌇";
        let symbol_horizontal = "─";
        let symbol_empty = " ";
        let symbol_merge_left_from = "⎨";
        let symbol_merge_right_from = "╭";
        let symbol_branch_up = "╯";
        let symbol_branch_down = "╮";
        let symbol_merge = "•";

        // Layers
        let mut layers: HashMap<Layers, Vec<(String, Color)>> = HashMap::new();
        let mut layer = |color: &ColorPicker, layer: Layers, symbol: String, lane: usize, custom: Option<Color>| {
            layers.entry(layer).or_default().push((symbol, custom.unwrap_or(color.get(lane))));
        };

        {
            // Otherwise (meaning we reached a tip, merge or a non-branching commit)
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for metadata in &_buffer {
                if metadata.sha == Oid::zero() {
                   if let Some(prev) = _buffer_prev.get(lane_idx) {
                        if prev.parents.len() == 1 {
                            layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                            layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                            layer(&color, Layers::Pipes, symbol_branch_up.to_string(), lane_idx, None);
                            layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                        } else {
                            layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                            layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                            layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                            layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                        }
                    }
                } else if sha == metadata.sha {
                    is_commit_found = true;

                    if metadata.parents.len() > 1 && !_tips.contains_key(&sha) {
                        layer(&color, Layers::Commits, symbol_merge.to_string(), lane_idx, None);
                    } else {
                        if _tips.contains_key(&sha) {
                            color.alternate(lane_idx);
                            _tip_colors.insert(sha, color.get(lane_idx));
                            layer(&color, Layers::Commits, symbol_commit_branch.to_string(), lane_idx, None);
                        } else {
                            layer(&color, Layers::Commits, symbol_commit.to_string(), lane_idx, None);
                        };
                    }
                    layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                    layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                    layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);

                    // Check if commit is being merged into
                    let mut is_mergee_found = false;
                    let mut is_drawing = false;
                    if metadata.parents.len() > 1 {

                        let mut is_merger_found = false;
                        let mut merger_idx: usize = 0;
                        for mtdt in &_buffer {
                            if mtdt.parents.len() == 1 && metadata.parents.last().unwrap() == mtdt.parents.first().unwrap() {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for mtdt in &_buffer {
                            if sha == mtdt.sha { break }
                            mergee_idx += 1;
                        }

                        let mut mtdt_idx = 0;
                        for mtdt in &_buffer {
                            if !is_mergee_found {
                                if sha == mtdt.sha {
                                    is_mergee_found = true;
                                    if is_merger_found { is_drawing = !is_drawing; }
                                    if !is_drawing { is_merged_before = true;}
                                    layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                    layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                } else {
                                    // Before the commit
                                    if !is_merger_found {
                                        layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                        layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                    } else {
                                        if mtdt.parents.len() == 1 && metadata.parents.contains(&mtdt.parents.first().unwrap()) {
                                            layer(&color, Layers::Merges, symbol_merge_right_from.to_string(), merger_idx, None);
                                            if mtdt_idx + 1 == mergee_idx {
                                                layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                            } else {
                                                layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                            }
                                            is_drawing = true;
                                        } else {
                                            if is_drawing {
                                                if mtdt_idx + 1 == mergee_idx {
                                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                                    layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                                } else {
                                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                                }
                                            } else {
                                                layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                                layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // After the commit
                                if is_merger_found && !is_merged_before {
                                    if mtdt.parents.len() == 1 && metadata.parents.contains(&mtdt.parents.first().unwrap()) {
                                        // color.alternate(merger_idx);
                                        layer(&color, Layers::Merges, symbol_merge_left_from.to_string(), merger_idx, None);
                                        layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                        is_drawing = false;
                                    } else {
                                        if is_drawing {
                                            layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                            layer(&color, Layers::Merges, symbol_horizontal.to_string(), merger_idx, None);
                                        }
                                        else {
                                            layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                            layer(&color, Layers::Merges, symbol_empty.to_string(), merger_idx, None);
                                        }
                                    }
                                }
                            }

                            mtdt_idx += 1;
                        }

                        if !is_merger_found {

                            // Count how many dummies in the end to get the real last element, append there
                            let mut idx = _buffer.len() - 1;
                            let mut trailing_dummies = 0;
                            for (i, c) in _buffer.iter().enumerate().rev() {
                                if !c.is_dummy() {
                                    idx = i;
                                    break;
                                } else {
                                    trailing_dummies += 1;
                                }
                            }

                            if trailing_dummies > 0 && _buffer_prev.len() > idx && _buffer_prev[idx + 1].is_dummy() {
                                color.alternate(idx + 1);
                                layer(&color, Layers::Merges, symbol_branch_down.to_string(), idx + 1, None);
                                layer(&color, Layers::Merges, symbol_empty.to_string(), idx + 1, None);
                            } else if trailing_dummies > 0 {
                                // color.alternate(idx + 1);
                                
                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), idx + 1, None);
                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), idx + 1, None);
                                }
                                
                                layer(&color, Layers::Merges, symbol_merge_left_from.to_string(), idx + 1, None);
                                layer(&color, Layers::Merges, symbol_empty.to_string(), idx + 1, None);
                            } else {
                                color.alternate(idx + 1);
                                
                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), idx + 1, None);
                                    layer(&color, Layers::Merges, symbol_horizontal.to_string(), idx + 1, None);
                                }

                                layer(&color, Layers::Merges, symbol_branch_down.to_string(), idx + 1, None);
                                layer(&color, Layers::Merges, symbol_empty.to_string(), idx + 1, None);
                            }
                            _not_found_mergers.push(metadata.sha.clone());
                        }
                    }
                } else {
                    layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                    layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                    layer(&color, Layers::Pipes, if metadata.parents.contains(&head_sha) { symbol_vertical_dotted.to_string() } else { symbol_vertical.to_string() }, lane_idx, if metadata.parents.contains(&head_sha) { Some(COLOR_GREY_500) } else { None });
                    layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                }

                lane_idx += 1;
            }        
            if !is_commit_found {
                
                if _tips.contains_key(&sha) {
                    color.alternate(lane_idx);
                    _tip_colors.insert(sha, color.get(lane_idx));
                    layer(&color, Layers::Commits, symbol_commit_branch.to_string(), lane_idx, None);
                } else {
                    layer(&color, Layers::Commits, symbol_commit.to_string(), lane_idx, None);
                };
                
                layer(&color, Layers::Commits, symbol_empty.to_string(), lane_idx, None);
                layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
                layer(&color, Layers::Pipes, symbol_empty.to_string(), lane_idx, None);
            }
        }

        // Blend layers into the graph
        {
            // Determine max length across all layers
            let max_len = [Layers::Commits, Layers::Merges, Layers::Pipes]
                .iter()
                .filter_map(|layer| layers.get(layer))
                .map(|tokens| tokens.len())
                .max()
                .unwrap_or(0);

            // For each token
            for token_index in 0..max_len {
                let mut symbol = symbol_empty;
                let mut color: Color = Color::Black;

                // For each layer
                for layer in [ Layers::Commits, Layers::Merges, Layers::Pipes] {
                    if let Some(tokens) = layers.get(&layer) {
                        if token_index < tokens.len() {
                            // If the layer has a token at this index
                            if let Some((_symbol, _color)) = tokens.get(token_index) {
                                if _symbol.trim() != "" {
                                    symbol = _symbol;
                                    color = *_color;
                                    break;
                                }
                            }
                        }
                    }
                }
                spans_graph.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
            }
        }

        _buffer_prev = _buffer.clone();

        // Serialize
        serialize_shas(&sha, &mut shas);
        serialize_graph(&sha, &mut graph, spans_graph);
        serialize_branches(&sha, &mut branches, &_tips, &_tip_colors, &_branches, &commit);
        serialize_messages(&commit, &mut messages);
        serialize_buffer(&sha, &_buffer, &_timestamps, &mut buffer);
    }

    (shas, graph, branches, messages, buffer)
}

fn update_buffer(buffer: &mut Vec<CommitMetadata>, _not_found_mergers: &mut Vec<Oid>, metadata: CommitMetadata) {

    // Erase trailing dummy metadata
    while buffer.last().map_or(false, |c| c.is_dummy()) {
        buffer.pop();
    }

    // If we have a planned merge later on
    if let Some(merger_idx) = buffer.iter().position(|inner| {
        _not_found_mergers.iter().any(|sha| sha == &inner.sha)
    }) {
        // Find the index in `_not_found_mergers` of the matching SHA
        if let Some(merger_pos) = _not_found_mergers.iter().position(|sha| sha == &buffer[merger_idx].sha) {
            _not_found_mergers.remove(merger_pos);
        }

        // Clone the element at merger_idx
        let mut clone = buffer[merger_idx].clone();
        clone.parents.remove(0);

        // Remove second parent from the original
        buffer[merger_idx].parents.remove(1);

        // Insert it right after the found index
        buffer.push(clone);
    }
    
    // Replace or append buffer metadata    
    if let Some(first_idx) = buffer.iter().position(|inner| inner.parents.contains(&metadata.sha)) {

        let old_sha = metadata.sha;
        
        // Replace metadata
        buffer[first_idx] = metadata;
        let keep_ptr = buffer[first_idx].parents.as_ptr();

        // Place dummies in case of branching
        for inner in buffer.iter_mut() {
            if inner.parents.contains(&old_sha) && inner.parents.as_ptr() != keep_ptr {
                if inner.parents.len() > 1 {
                     inner.parents.retain(|sha| *sha != old_sha);
                } else {
                    *inner = CommitMetadata::dummy();
                }
            }
        }

    } else {
        buffer.push(metadata);
    }
}

fn get_sorted_commits(repo: &Repository) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();

    // Push all branch tips
    for branch in repo.branches(None).unwrap() {
        let (branch, _) = branch.unwrap();
        if let Some(oid) = branch.get().target() {
            revwalk.push(oid).unwrap();
        }
    }

    // Topological + chronological order
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME).unwrap();

    revwalk.filter_map(Result::ok).collect()
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

fn get_timestamps(repo: &Repository, _branches: &HashMap<Oid, Vec<String>>) -> HashMap<Oid, (Time, Time, Time)> {
    _branches
        .keys()
        .map(|&sha| {
            let commit = repo.find_commit(sha).unwrap();
            let author_time = commit.author().when();
            let committer_time = commit.committer().when();
            let time = commit.time();
            (sha, (time, committer_time, author_time))
        })
        .collect()
}

fn serialize_graph(sha: &Oid, graph: &mut Vec<Line<>>, spans_graph: Vec<Span<'static>>) {
    let span_sha = Span::styled(sha.to_string()[..7].to_string(), Style::default().fg(COLOR_TEXT));
    let mut spans = Vec::new();
    spans.push(span_sha);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

fn serialize_branches(sha: &Oid, branches: &mut Vec<Line<>>, _tips: &HashMap<Oid, Vec<String>>, _tip_colors: &HashMap<Oid, Color>, _branches: &HashMap<Oid, Vec<String>>, commit: &Commit<'_>) {
    let mut spans = Vec::new();
    let span_tips: Vec<Span<'_>> = _tips.get(&sha).map(|branches| {
        branches.iter().flat_map(|branch| {
            vec![
                Span::styled(format!("● {} ", branch), Style::default().fg(*_tip_colors.get(sha).unwrap_or(&Color::White))),
            ]
            
        }).collect()
    }).unwrap_or_default();
    spans.extend(span_tips);
    
    let span_message = Span::styled(commit.summary().unwrap_or("<no message>").to_string(), Style::default().fg(COLOR_TEXT));
    spans.push(span_message);
    // let span_branches = Span::styled(_branches.get(&sha).unwrap().join(","), Style::default().fg(Color::Yellow));
    // spans.push(span_branches);
    branches.push(Line::from(spans));
}

fn serialize_messages(commit: &Commit<'_>, messages: &mut Vec<Line<>>) {
    let mut spans = Vec::new();
    let span_message = Span::styled(commit.summary().unwrap_or("<no message>").to_string(), Style::default().fg(COLOR_TEXT));
    spans.push(span_message);
    messages.push(Line::from(spans));
}

fn serialize_shas(sha: &Oid, shas: &mut Vec<Oid<>>) {
    shas.push(sha.clone());
}

fn serialize_buffer(_sha: &Oid, _buffer: &Vec<CommitMetadata>, _timestamps: &HashMap<Oid, (Time, Time, Time)>, buffer: &mut Vec<Line<>>) {
    let mut _spans = Vec::new();

    // let time = _timestamps.get(_sha).unwrap().0.seconds();
    // let o_time = _timestamps.get(_sha).unwrap().0.offset_minutes();
    // let committer_time = _timestamps.get(_sha).unwrap().1.seconds();
    // let o_committer_time = _timestamps.get(_sha).unwrap().1.offset_minutes();
    // let author_time = _timestamps.get(_sha).unwrap().1.seconds();
    // let o_author_time = _timestamps.get(_sha).unwrap().1.offset_minutes();
    // let span_timestamp = Span::styled(format!("{}:{:.3}:{}:{:.3}:{}:{:.3} ", time, o_time, committer_time, o_committer_time, author_time, o_author_time), Style::default().fg(Color::DarkGray));
    // spans.push(span_timestamp);
    
    // let formatted_buffer: String = _buffer.iter().map(|metadata| {
    //         format!(
    //             "{:.2}({:<5})",
    //             metadata.sha,
    //             if metadata.parents.len() > 0 {
    //                 let a = metadata.parents.iter().map(|oid| {format!("{:.2}", oid)}).collect::<Vec<String>>();
    //                 let mut s = a.join(",");
    //                 if a.len() == 1 {
    //                     s.push(',');
    //                     s.push('-');
    //                     s.push('-');
    //                 }
    //                 s
    //             } else {"--,--".to_string()},
    //         )
    //     }).collect::<Vec<String>>().join(" ");
    // let span_buffer = Span::styled(formatted_buffer, Style::default().fg(COLOR_TEXT));
    // spans.push(span_buffer);

    buffer.push(Line::from(_spans));
}

pub fn get_uncommitted_changes(repo: &Repository) -> Vec<String> {
    let mut options = StatusOptions::new();
    options.include_untracked(true); // include untracked files
    options.include_ignored(false);  // ignore ignored files
    options.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut options)).unwrap();

    let mut changed_files = Vec::new();

    for entry in statuses.iter() {
        let s = entry.status();
        let path = entry.path().unwrap_or("<unknown>").to_string();

        if s.contains(Status::WT_NEW) || s.contains(Status::WT_MODIFIED) || s.contains(Status::WT_DELETED) {
            changed_files.push(path);
        }

        // staged changes
        // if s.contains(Status::INDEX_NEW) || s.contains(Status::INDEX_MODIFIED) || s.contains(Status::INDEX_DELETED) {
        //     changed_files.push(path);
        // }
    }

    changed_files
}

pub fn get_changed_filenames_text(repo: &Repository, oid: Oid) -> Text<'_> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();

    let mut lines = Vec::new();

    if commit.parent_count() == 0 {
        // Initial commit — list all files
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                lines.push(Line::from(Span::styled(
                    name.to_string(),
                    Style::default().fg(COLOR_GREY_400),
                )));
            }
            git2::TreeWalkResult::Ok
        }).unwrap();
    } else {
        // Normal commits — diff against first parent
        let parent = commit.parent(0).unwrap();
        let parent_tree = parent.tree().unwrap();
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None).unwrap();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    lines.push(Line::from(Span::styled(
                        path.display().to_string(),
                        Style::default().fg(
                            match delta.status() {
                                Delta::Added => COLOR_GREEN,
                                Delta::Deleted => COLOR_RED,
                                Delta::Modified => COLOR_TEXT,
                                Delta::Renamed => COLOR_TEXT,
                                Delta::Copied => COLOR_TEXT,
                                Delta::Untracked => COLOR_TEXT,
                                _ => COLOR_TEXT,
                            }
                        ),
                    )));
                }
                true
            },
            None,
            None,
            None,
        ).unwrap();
    }

    Text::from(lines)
}

pub fn timestamp_to_utc(time: Time) -> String {
    // Create a DateTime with the given offset
    let offset = FixedOffset::east_opt(time.offset_minutes() * 60).unwrap();
    
    // Create UTC datetime from timestamp
    let utc_datetime = DateTime::from_timestamp(time.seconds(), 0)
        .expect("Invalid timestamp");
    
    // Convert to local time with offset, then back to UTC
    let local_datetime = offset.from_utc_datetime(&utc_datetime.naive_utc());
    let final_utc: DateTime<Utc> = local_datetime.with_timezone(&Utc);
    
    // Format as string
    final_utc.to_rfc2822()
}
