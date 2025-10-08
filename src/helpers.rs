use crate::{
    core::{buffer::Buffer, chunk::Chunk, layers::LayersCtx},
    git::queries::{
        get_branches, get_sorted_commits, get_timestamps, get_tips, get_uncommitted_changes_count,
    },
    layers,
    utils::{
        colors::*,
        symbols::{
            SYM_BRANCH_DOWN, SYM_BRANCH_UP, SYM_COMMIT, SYM_COMMIT_BRANCH, SYM_EMPTY,
            SYM_HORIZONTAL, SYM_MERGE, SYM_MERGE_LEFT_FROM, SYM_MERGE_RIGHT_FROM, SYM_UNCOMMITED,
            SYM_VERTICAL, SYM_VERTICAL_DOTTED,
        },
    },
};
use git2::{Commit, Oid, Repository, Time};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
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
    let _buffer = RefCell::new(Buffer::default());

    let mut graph = Vec::new();
    let mut branches = Vec::new();
    let mut messages = Vec::new();
    let mut buffer = Vec::new();
    let mut shas = Vec::new();

    let _tips: HashMap<Oid, Vec<String>> = get_tips(&repo);
    let mut _tip_colors: HashMap<Oid, Color> = HashMap::new();
    let _branches: HashMap<Oid, Vec<String>> = get_branches(&repo, &_tips);
    let _timestamps: HashMap<Oid, (Time, Time, Time)> = get_timestamps(&repo, &_branches);
    let mut _sorted: Vec<Oid> = get_sorted_commits(&repo);
    let mut layers: LayersCtx = layers!(&color);

    // Make a fake commit for unstaged changes
    let (new_count, modified_count, deleted_count) = get_uncommitted_changes_count(repo);
    let head = repo.head().unwrap();
    let head_sha = head.target().unwrap();
    {
        shas.push(Oid::zero());
        let mut uncommited_line_spans = vec![Span::styled(
            format!("{} ", SYM_UNCOMMITED),
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
            Span::styled(SYM_UNCOMMITED, Style::default().fg(COLOR_GREY_400)),
        ]));
        // _buffer.push(value);
        let parents: Vec<Oid> = vec![head_sha];
        let metadata = Chunk {
            sha: Oid::from_str("0000000000000000000000000000000000000001").unwrap(),
            parents,
        };

        // Update
        _buffer.borrow_mut().update(metadata);
    }

    // Go through the commits, inferring the graph
    for sha in _sorted {
        let mut merger_sha = None;

        layers.clear();
        let commit = repo.find_commit(sha).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let metadata = Chunk { sha, parents };

        let mut spans_graph = Vec::new();

        // Update
        _buffer.borrow_mut().update(metadata);

        {
            // Otherwise (meaning we reached a tip, merge or a non-branching commit)
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for metadata in &_buffer.borrow().curr {
                if metadata.sha == Oid::zero() {
                    if let Some(prev) = _buffer.borrow().prev.get(lane_idx) {
                        if prev.parents.len() == 1 {
                            layers.commit(SYM_EMPTY, lane_idx);
                            layers.commit(SYM_EMPTY, lane_idx);
                            layers.pipe(SYM_BRANCH_UP, lane_idx);
                            layers.pipe(SYM_EMPTY, lane_idx);
                        } else {
                            layers.commit(SYM_EMPTY, lane_idx);
                            layers.commit(SYM_EMPTY, lane_idx);
                            layers.pipe(SYM_EMPTY, lane_idx);
                            layers.pipe(SYM_EMPTY, lane_idx);
                        }
                    }
                } else if sha == metadata.sha {
                    is_commit_found = true;

                    if metadata.parents.len() > 1 && !_tips.contains_key(&sha) {
                        layers.commit(SYM_MERGE, lane_idx);
                    } else {
                        if _tips.contains_key(&sha) {
                            color.borrow_mut().alternate(lane_idx);
                            _tip_colors.insert(sha, color.borrow().get(lane_idx));
                            layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                        } else {
                            layers.commit(SYM_COMMIT, lane_idx);
                        };
                    }
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.pipe(SYM_EMPTY, lane_idx);
                    layers.pipe(SYM_EMPTY, lane_idx);

                    // Check if commit is being merged into
                    let mut is_mergee_found = false;
                    let mut is_drawing = false;
                    if metadata.parents.len() > 1 {
                        let mut is_merger_found = false;
                        let mut merger_idx: usize = 0;
                        for mtdt in &_buffer.borrow().curr {
                            if mtdt.parents.len() == 1
                                && metadata.parents.last().unwrap() == mtdt.parents.first().unwrap()
                            {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for mtdt in &_buffer.borrow().curr {
                            if sha == mtdt.sha {
                                break;
                            }
                            mergee_idx += 1;
                        }

                        let mut mtdt_idx = 0;
                        for mtdt in &_buffer.borrow().curr {
                            if !is_mergee_found {
                                if sha == mtdt.sha {
                                    is_mergee_found = true;
                                    if is_merger_found {
                                        is_drawing = !is_drawing;
                                    }
                                    if !is_drawing {
                                        is_merged_before = true;
                                    }
                                    layers.merge(SYM_EMPTY, merger_idx);
                                    layers.merge(SYM_EMPTY, merger_idx);
                                } else {
                                    // Before the commit
                                    if !is_merger_found {
                                        layers.merge(SYM_EMPTY, merger_idx);
                                        layers.merge(SYM_EMPTY, merger_idx);
                                    } else {
                                        if mtdt.parents.len() == 1
                                            && metadata
                                                .parents
                                                .contains(&mtdt.parents.first().unwrap())
                                        {
                                            layers.merge(SYM_MERGE_RIGHT_FROM, merger_idx);
                                            if mtdt_idx + 1 == mergee_idx {
                                                layers.merge(SYM_EMPTY, merger_idx);
                                            } else {
                                                layers.merge(SYM_HORIZONTAL, merger_idx);
                                            }
                                            is_drawing = true;
                                        } else {
                                            if is_drawing {
                                                if mtdt_idx + 1 == mergee_idx {
                                                    layers.merge(SYM_HORIZONTAL, merger_idx);
                                                    layers.merge(SYM_EMPTY, merger_idx);
                                                } else {
                                                    layers.merge(SYM_HORIZONTAL, merger_idx);
                                                    layers.merge(SYM_HORIZONTAL, merger_idx);
                                                }
                                            } else {
                                                layers.merge(SYM_EMPTY, merger_idx);
                                                layers.merge(SYM_EMPTY, merger_idx);
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
                                        layers.merge(SYM_MERGE_LEFT_FROM, merger_idx);
                                        layers.merge(SYM_EMPTY, merger_idx);
                                        is_drawing = false;
                                    } else {
                                        if is_drawing {
                                            layers.merge(SYM_HORIZONTAL, merger_idx);
                                            layers.merge(SYM_HORIZONTAL, merger_idx);
                                        } else {
                                            layers.merge(SYM_EMPTY, merger_idx);
                                            layers.merge(SYM_EMPTY, merger_idx);
                                        }
                                    }
                                }
                            }

                            mtdt_idx += 1;
                        }

                        if !is_merger_found {
                            // Count how many dummies in the end to get the real last element, append there
                            let mut idx = _buffer.borrow().curr.len() - 1;
                            let mut trailing_dummies = 0;
                            for (i, c) in _buffer.borrow().curr.iter().enumerate().rev() {
                                if !c.is_dummy() {
                                    idx = i;
                                    break;
                                } else {
                                    trailing_dummies += 1;
                                }
                            }

                            if trailing_dummies > 0
                                && _buffer.borrow().prev.len() > idx
                                && _buffer.borrow().prev[idx + 1].is_dummy()
                            {
                                color.borrow_mut().alternate(idx + 1);
                                layers.merge(SYM_BRANCH_DOWN, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            } else if trailing_dummies > 0 {
                                // color.alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                }

                                layers.merge(SYM_MERGE_LEFT_FROM, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            } else {
                                color.borrow_mut().alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                }

                                layers.merge(SYM_BRANCH_DOWN, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            }
                            merger_sha = Some(metadata.sha);
                        }
                    }
                } else {
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.commit(SYM_EMPTY, lane_idx);
                    if metadata.parents.contains(&head_sha) && lane_idx == 0 {
                        layers.pipe_custom(SYM_VERTICAL_DOTTED, lane_idx, COLOR_GREY_500);
                    } else {
                        layers.pipe(SYM_VERTICAL, lane_idx);
                    }
                    layers.pipe(SYM_EMPTY, lane_idx);
                }

                lane_idx += 1;
            }
            if !is_commit_found {
                if _tips.contains_key(&sha) {
                    color.borrow_mut().alternate(lane_idx);
                    _tip_colors.insert(sha, color.borrow().get(lane_idx));
                    layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                } else {
                    layers.commit(SYM_COMMIT, lane_idx);
                };
                layers.commit(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);
            }
        }

        // Blend layers into the graph
        layers.bake(&mut spans_graph);

        // Now we can borrow mutably
        if let Some(sha) = merger_sha {
            _buffer.borrow_mut().merger(sha);
        }
        _buffer.borrow_mut().backup();

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
    _buffer: &RefCell<Buffer>,
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
    // _spans.push(span_timestamp);

    // let formatted_buffer: String = _buffer.curr.iter().map(|metadata| {
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
    // _spans.push(span_buffer);

    buffer.push(Line::from(_spans));
}
