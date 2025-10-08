use crate::{
    colors::*,
    graph::{
        chunk::Chunk,
        layers::{LayerTypes, LayersCtx},
    },
    layers,
};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use git2::{
    BranchType, Commit, Delta, Oid, Repository, Status, StatusOptions, Time, build::CheckoutBuilder,
};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};
use std::{cell::RefCell, collections::HashMap};

pub fn get_commits(
    repo: &Repository,
) -> (
    Vec<Oid>,
    Vec<Line<'static>>,
    Vec<Line<'static>>,
    Vec<Line<'static>>,
    Vec<Line<'static>>,
    HashMap<Oid, Vec<String>>,
) {
    let color = RefCell::new(ColorPicker::default());
    let mut graph = Vec::new();
    let mut branches = Vec::new();
    let mut messages = Vec::new();
    let mut buffer = Vec::new();
    let mut shas = Vec::new();

    let mut _buffer_prev: Vec<Chunk> = Vec::new();
    let mut _buffer: Vec<Chunk> = Vec::new();
    let _tips: HashMap<Oid, Vec<String>> = get_tips(&repo);
    let mut _tip_colors: HashMap<Oid, Color> = HashMap::new();
    let _branches: HashMap<Oid, Vec<String>> = get_branches(&repo, &_tips);
    let _timestamps: HashMap<Oid, (Time, Time, Time)> = get_timestamps(&repo, &_branches);
    let mut _sorted: Vec<Oid> = get_sorted_commits(&repo);
    let mut _not_found_mergers: Vec<Oid> = Vec::new();
    let mut layers: LayersCtx = layers!(&color);

    // Make a fake commit for unstaged changes
    let (new_count, modified_count, deleted_count) = get_uncommitted_changes_counts(repo);
    let head = repo.head().unwrap();
    let head_sha = head.target().unwrap();
    {
        shas.push(Oid::zero());
        let mut uncommited_line_spans = vec![Span::styled(
            format!("◌ "),
            Style::default().fg(COLOR_GREY_400),
        )];
        if modified_count > 0 {
            uncommited_line_spans.push(Span::styled(
                format!("~{} ", modified_count),
                Style::default().fg(COLOR_GREY_400),
            ));
        }
        if new_count > 0 {
            uncommited_line_spans.push(Span::styled(
                format!("+{} ", new_count),
                Style::default().fg(COLOR_GREY_400),
            ));
        }
        if new_count > 0 {
            uncommited_line_spans.push(Span::styled(
                format!("-{} ", deleted_count),
                Style::default().fg(COLOR_GREY_400),
            ));
        }

        branches.push(Line::from(uncommited_line_spans));
        buffer.push(Line::from(Span::styled(
            "--",
            Style::default().fg(COLOR_GREY_400),
        )));
        graph.push(Line::from(vec![
            Span::styled("•••••• ", Style::default().fg(COLOR_TEXT)),
            Span::styled("◌", Style::default().fg(COLOR_GREY_400)),
        ]));
        // _buffer.push(value);
        let parents: Vec<Oid> = vec![head_sha];
        let metadata = Chunk {
            sha: Oid::from_str("0000000000000000000000000000000000000001").unwrap(),
            parents,
        };

        // Update
        update_buffer(&mut _buffer, &mut _not_found_mergers, metadata);
    }

    // Go through the commits, inferring the graph
    for sha in _sorted {
        layers.clear();
        let commit = repo.find_commit(sha).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let metadata = Chunk { sha, parents };

        let mut spans_graph = Vec::new();

        // Update
        update_buffer(&mut _buffer, &mut _not_found_mergers, metadata);

        // Symbols
        let symbol_commit_branch = "●";
        let symbol_commit = "○";
        let symbol_vertical = "│";
        let symbol_vertical_dotted = "┊";
        let symbol_horizontal = "─";
        let symbol_empty = " ";
        let symbol_merge_left_from = "⎨";
        let symbol_merge_right_from = "╭";
        let symbol_branch_up = "╯";
        let symbol_branch_down = "╮";
        let symbol_merge = "•";

        {
            // Otherwise (meaning we reached a tip, merge or a non-branching commit)
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for metadata in &_buffer {
                if metadata.sha == Oid::zero() {
                    if let Some(prev) = _buffer_prev.get(lane_idx) {
                        if prev.parents.len() == 1 {
                            layers.commit(symbol_empty, lane_idx);
                            layers.commit(symbol_empty, lane_idx);
                            layers.pipe(symbol_branch_up, lane_idx);
                            layers.pipe(symbol_empty, lane_idx);
                        } else {
                            layers.commit(symbol_empty, lane_idx);
                            layers.commit(symbol_empty, lane_idx);
                            layers.pipe(symbol_empty, lane_idx);
                            layers.pipe(symbol_empty, lane_idx);
                        }
                    }
                } else if sha == metadata.sha {
                    is_commit_found = true;

                    if metadata.parents.len() > 1 && !_tips.contains_key(&sha) {
                        layers.commit(symbol_merge, lane_idx);
                    } else {
                        if _tips.contains_key(&sha) {
                            color.borrow_mut().alternate(lane_idx);
                            _tip_colors.insert(sha, color.borrow().get(lane_idx));
                            layers.commit(symbol_commit_branch, lane_idx);
                        } else {
                            layers.commit(symbol_commit, lane_idx);
                        };
                    }
                    layers.commit(symbol_empty, lane_idx);
                    layers.pipe(symbol_empty, lane_idx);
                    layers.pipe(symbol_empty, lane_idx);

                    // Check if commit is being merged into
                    let mut is_mergee_found = false;
                    let mut is_drawing = false;
                    if metadata.parents.len() > 1 {
                        let mut is_merger_found = false;
                        let mut merger_idx: usize = 0;
                        for mtdt in &_buffer {
                            if mtdt.parents.len() == 1
                                && metadata.parents.last().unwrap() == mtdt.parents.first().unwrap()
                            {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for mtdt in &_buffer {
                            if sha == mtdt.sha {
                                break;
                            }
                            mergee_idx += 1;
                        }

                        let mut mtdt_idx = 0;
                        for mtdt in &_buffer {
                            if !is_mergee_found {
                                if sha == mtdt.sha {
                                    is_mergee_found = true;
                                    if is_merger_found {
                                        is_drawing = !is_drawing;
                                    }
                                    if !is_drawing {
                                        is_merged_before = true;
                                    }
                                    layers.merge(symbol_empty, merger_idx);
                                    layers.merge(symbol_empty, merger_idx);
                                } else {
                                    // Before the commit
                                    if !is_merger_found {
                                        layers.merge(symbol_empty, merger_idx);
                                        layers.merge(symbol_empty, merger_idx);
                                    } else {
                                        if mtdt.parents.len() == 1
                                            && metadata
                                                .parents
                                                .contains(&mtdt.parents.first().unwrap())
                                        {
                                            layers.merge(symbol_merge_right_from, merger_idx);
                                            if mtdt_idx + 1 == mergee_idx {
                                                layers.merge(symbol_empty, merger_idx);
                                            } else {
                                                layers.merge(symbol_horizontal, merger_idx);
                                            }
                                            is_drawing = true;
                                        } else {
                                            if is_drawing {
                                                if mtdt_idx + 1 == mergee_idx {
                                                    layers.merge(symbol_horizontal, merger_idx);
                                                    layers.merge(symbol_empty, merger_idx);
                                                } else {
                                                    layers.merge(symbol_horizontal, merger_idx);
                                                    layers.merge(symbol_horizontal, merger_idx);
                                                }
                                            } else {
                                                layers.merge(symbol_empty, merger_idx);
                                                layers.merge(symbol_empty, merger_idx);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // After the commit
                                if is_merger_found && !is_merged_before {
                                    if mtdt.parents.len() == 1
                                        && metadata.parents.contains(mtdt.parents.first().unwrap())
                                    {
                                        layers.merge(symbol_merge_left_from, merger_idx);
                                        layers.merge(symbol_empty, merger_idx);
                                        is_drawing = false;
                                    } else {
                                        if is_drawing {
                                            layers.merge(symbol_horizontal, merger_idx);
                                            layers.merge(symbol_horizontal, merger_idx);
                                        } else {
                                            layers.merge(symbol_empty, merger_idx);
                                            layers.merge(symbol_empty, merger_idx);
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

                            if trailing_dummies > 0
                                && _buffer_prev.len() > idx
                                && _buffer_prev[idx + 1].is_dummy()
                            {
                                color.borrow_mut().alternate(idx + 1);
                                layers.merge(symbol_branch_down, idx + 1);
                                layers.merge(symbol_empty, idx + 1);
                            } else if trailing_dummies > 0 {
                                // color.alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(symbol_horizontal, idx + 1);
                                    layers.merge(symbol_horizontal, idx + 1);
                                }

                                layers.merge(symbol_merge_left_from, idx + 1);
                                layers.merge(symbol_empty, idx + 1);
                            } else {
                                color.borrow_mut().alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(symbol_horizontal, idx + 1);
                                    layers.merge(symbol_horizontal, idx + 1);
                                }

                                layers.merge(symbol_branch_down, idx + 1);
                                layers.merge(symbol_empty, idx + 1);
                            }
                            _not_found_mergers.push(metadata.sha);
                        }
                    }
                } else {
                    layers.commit(symbol_empty, lane_idx);
                    layers.commit(symbol_empty, lane_idx);
                    if metadata.parents.contains(&head_sha) && lane_idx == 0 {
                        layers.pipe_custom(symbol_vertical_dotted, lane_idx, COLOR_GREY_500);
                    } else {
                        layers.pipe(symbol_vertical, lane_idx);
                    }
                    layers.pipe(symbol_empty, lane_idx);
                }

                lane_idx += 1;
            }
            if !is_commit_found {
                if _tips.contains_key(&sha) {
                    color.borrow_mut().alternate(lane_idx);
                    _tip_colors.insert(sha, color.borrow().get(lane_idx));
                    layers.commit(symbol_commit_branch, lane_idx);
                } else {
                    layers.commit(symbol_commit, lane_idx);
                };
                layers.commit(symbol_empty, lane_idx);
                layers.pipe(symbol_empty, lane_idx);
                layers.pipe(symbol_empty, lane_idx);
            }
        }

        // Blend layers into the graph
        layers.bake(&mut spans_graph);

        _buffer_prev = _buffer.clone();

        // Serialize
        serialize_shas(&sha, &mut shas);
        serialize_graph(&sha, &mut graph, spans_graph);
        serialize_branches(
            &sha,
            &mut branches,
            &_tips,
            &_tip_colors,
            &_branches,
            &commit,
        );
        serialize_messages(&commit, &mut messages);
        serialize_buffer(&sha, &_buffer, &_timestamps, &mut buffer);
    }

    (shas, graph, branches, messages, buffer, _tips)
}

fn update_buffer(buffer: &mut Vec<Chunk>, _not_found_mergers: &mut Vec<Oid>, metadata: Chunk) {
    // Erase trailing dummy metadata
    while buffer.last().is_some_and(|c| c.is_dummy()) {
        buffer.pop();
    }

    // If we have a planned merge later on
    if let Some(merger_idx) = buffer
        .iter()
        .position(|inner| _not_found_mergers.iter().any(|sha| sha == &inner.sha))
    {
        // Find the index in `_not_found_mergers` of the matching SHA
        if let Some(merger_pos) = _not_found_mergers
            .iter()
            .position(|sha| sha == &buffer[merger_idx].sha)
        {
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
    if let Some(first_idx) = buffer
        .iter()
        .position(|inner| inner.parents.contains(&metadata.sha))
    {
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
                    *inner = Chunk::dummy();
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
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
        .unwrap();

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

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().unwrap();
    if head.is_branch() {
        head.shorthand().map(|s| s.to_string())
    } else {
        None
    }
}

fn get_timestamps(
    repo: &Repository,
    _branches: &HashMap<Oid, Vec<String>>,
) -> HashMap<Oid, (Time, Time, Time)> {
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

fn serialize_graph(sha: &Oid, graph: &mut Vec<Line>, spans_graph: Vec<Span<'static>>) {
    let span_sha = Span::styled(sha.to_string()[..6].to_string(), COLOR_TEXT);
    let mut spans = Vec::new();
    spans.push(span_sha);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

fn serialize_branches(
    sha: &Oid,
    branches: &mut Vec<Line>,
    _tips: &HashMap<Oid, Vec<String>>,
    _tip_colors: &HashMap<Oid, Color>,
    _branches: &HashMap<Oid, Vec<String>>,
    commit: &Commit<'_>,
) {
    let mut spans = Vec::new();
    let span_tips: Vec<Span<'_>> = _tips
        .get(sha)
        .map(|branches| {
            branches
                .iter()
                .flat_map(|branch| {
                    vec![Span::styled(
                        format!("● {} ", branch),
                        Style::default().fg(*_tip_colors.get(sha).unwrap_or(&Color::White)),
                    )]
                })
                .collect()
        })
        .unwrap_or_default();
    spans.extend(span_tips);

    let span_message = Span::styled(
        commit.summary().unwrap_or("<no message>").to_string(),
        Style::default().fg(COLOR_TEXT),
    );
    spans.push(span_message);
    // let span_branches = Span::styled(_branches.get(&sha).unwrap().join(","), Style::default().fg(Color::Yellow));
    // spans.push(span_branches);
    branches.push(Line::from(spans));
}

fn serialize_messages(commit: &Commit<'_>, messages: &mut Vec<Line>) {
    let mut spans = Vec::new();
    let span_message = Span::styled(
        commit.summary().unwrap_or("<no message>").to_string(),
        Style::default().fg(COLOR_TEXT),
    );
    spans.push(span_message);
    messages.push(Line::from(spans));
}

fn serialize_shas(sha: &Oid, shas: &mut Vec<Oid>) {
    shas.push(*sha);
}

fn serialize_buffer(
    _sha: &Oid,
    _buffer: &Vec<Chunk>,
    _timestamps: &HashMap<Oid, (Time, Time, Time)>,
    buffer: &mut Vec<Line>,
) {
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

pub fn get_uncommitted_changes_counts(repo: &Repository) -> (usize, usize, usize) {
    let mut options = StatusOptions::new();
    options.include_untracked(true); // include untracked files
    options.include_ignored(false); // skip ignored files
    options.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut options)).unwrap();

    let mut new_count = 0;
    let mut modified_count = 0;
    let mut deleted_count = 0;

    for entry in statuses.iter() {
        let status = entry.status();
        if status.contains(Status::WT_NEW) || status.contains(Status::INDEX_NEW) {
            new_count += 1;
        }
        if status.contains(Status::WT_MODIFIED) || status.contains(Status::INDEX_MODIFIED) {
            modified_count += 1;
        }
        if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
            deleted_count += 1;
        }
    }

    (new_count, modified_count, deleted_count)
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
        })
        .unwrap();
    } else {
        // Normal commits — diff against first parent
        let parent = commit.parent(0).unwrap();
        let parent_tree = parent.tree().unwrap();
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .unwrap();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    lines.push(Line::from(Span::styled(
                        path.display().to_string(),
                        Style::default().fg(match delta.status() {
                            Delta::Added => COLOR_GREEN,
                            Delta::Deleted => COLOR_RED,
                            Delta::Modified => COLOR_TEXT,
                            Delta::Renamed => COLOR_TEXT,
                            Delta::Copied => COLOR_TEXT,
                            Delta::Untracked => COLOR_TEXT,
                            _ => COLOR_TEXT,
                        }),
                    )));
                }
                true
            },
            None,
            None,
            None,
        )
        .unwrap();
    }

    Text::from(lines)
}

pub fn timestamp_to_utc(time: Time) -> String {
    // Create a DateTime with the given offset
    let offset = FixedOffset::east_opt(time.offset_minutes() * 60).unwrap();

    // Create UTC datetime from timestamp
    let utc_datetime = DateTime::from_timestamp(time.seconds(), 0).expect("Invalid timestamp");

    // Convert to local time with offset, then back to UTC
    let local_datetime = offset.from_utc_datetime(&utc_datetime.naive_utc());
    let final_utc: DateTime<Utc> = local_datetime.with_timezone(&Utc);

    // Format as string
    final_utc.to_rfc2822()
}

pub fn checkout_sha(repo: &Repository, sha: Oid) {
    // Find the commit object
    let commit = repo.find_commit(sha).unwrap();

    // Set HEAD to the commit (detached)
    repo.set_head_detached(commit.id()).unwrap();

    // Checkout the commit
    repo.checkout_head(Some(
        CheckoutBuilder::default().allow_conflicts(true).force(), // optional: force overwrite local changes
    ))
    .expect("Error checking out");
}
