#[rustfmt::skip]
use std::{
    cell::RefCell,
    collections::HashMap
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository,
    // Time
};
#[rustfmt::skip]
use ratatui::{
    style::{
        Color,
        Style
    },
    text::{
        Line,
        Span
    },
};
#[rustfmt::skip]
use crate::{
    core::{
        buffer::Buffer,
        chunk::Chunk,
        layers::LayersCtx,
        renderers::{
            render_branches,
            render_buffer,
            render_graph,
            render_messages
        },
    },
    git::queries::{
        // get_branches,
        get_sorted_commits,
        // get_timestamps,
        get_tips,
        get_uncommitted_changes_count,
    },
    utils::{
        colors::*,
        symbols::*
    },
    layers,
};

pub struct Walked<'a> {
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub lines_graph: Vec<Line<'a>>,
    pub lines_branches: Vec<Line<'a>>,
    pub lines_messages: Vec<Line<'a>>,
    pub lines_buffer: Vec<Line<'a>>,
}

pub fn walk(repo: &Repository) -> Walked<'static> {
    // Utilities
    let color = RefCell::new(ColorPicker::default());
    let buffer = RefCell::new(Buffer::default());
    let mut layers: LayersCtx = layers!(&color);

    // Renders
    let mut lines_graph = Vec::new();
    let mut lines_branches = Vec::new();
    let mut lines_messages = Vec::new();
    let mut lines_buffer = Vec::new();

    // Git state descriptors

    // Topologically sorted list of oids including the uncommited, for the sake of order
    let mut oids = Vec::new();
    // Mapping of tip oids of the branches to the branch names
    let tips: HashMap<Oid, Vec<String>> = get_tips(repo);
    // Mapping of tip oids of the branches to the colors
    let mut tip_colors: HashMap<Oid, Color> = HashMap::new();
    // Mapping of every oid to every branch it is a part of
    // let branches: HashMap<Oid, Vec<String>> = get_branches(repo, &tips);
    // Timestamps of every oid
    // let timestamps: HashMap<Oid, (Time, Time, Time)> = get_timestamps(repo, &branches);
    // Topologically sorted list of oids including the uncommited
    let sorted: Vec<Oid> = get_sorted_commits(repo);

    // Make a fake commit for unstaged changes
    let (new_count, modified_count, deleted_count) = get_uncommitted_changes_count(repo);
    let head = repo.head().unwrap();
    let head_oid = head.target().unwrap();
    {
        oids.push(Oid::zero());
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

        let metadata = Chunk::uncommitted(vec![head_oid]);
        lines_branches.push(Line::from(uncommited_line_spans));
        lines_buffer.push(Line::from(format!("UU({:.2},--)", head_oid)));
        lines_graph.push(Line::from(vec![
            Span::styled("······ ", Style::default().fg(COLOR_TEXT)),
            Span::styled(SYM_UNCOMMITED, Style::default().fg(COLOR_GREY_400)),
        ]));

        // Update
        buffer.borrow_mut().update(metadata);
    }

    // Go through the commits, inferring the graph
    for oid in sorted {
        let mut merger_oid = None;

        layers.clear();
        let commit = repo.find_commit(oid).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let metadata = Chunk::commit(oid, parents);

        let mut spans_graph = Vec::new();

        // Update
        buffer.borrow_mut().update(metadata);

        {
            // Otherwise (meaning we reached a tip, merge or a non-branching commit)
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for metadata in &buffer.borrow().curr {
                if metadata.is_dummy() {
                    if let Some(prev) = buffer.borrow().prev.get(lane_idx) {
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
                } else if oid == metadata.oid {
                    is_commit_found = true;

                    if metadata.parents.len() > 1 && !tips.contains_key(&oid) {
                        layers.commit(SYM_MERGE, lane_idx);
                    } else if tips.contains_key(&oid) {
                        color.borrow_mut().alternate(lane_idx);
                        tip_colors.insert(oid, color.borrow().get(lane_idx));
                        layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                    } else {
                        layers.commit(SYM_COMMIT, lane_idx);
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
                        for mtdt in &buffer.borrow().curr {
                            if mtdt.parents.len() == 1
                                && metadata.parents.last().unwrap() == mtdt.parents.first().unwrap()
                            {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for mtdt in &buffer.borrow().curr {
                            if oid == mtdt.oid {
                                break;
                            }
                            mergee_idx += 1;
                        }

                        for (mtdt_idx, mtdt) in buffer.borrow().curr.iter().enumerate() {
                            if !is_mergee_found {
                                if oid == mtdt.oid {
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
                                    } else if mtdt.parents.len() == 1
                                        && metadata.parents.contains(mtdt.parents.first().unwrap())
                                    {
                                        layers.merge(SYM_MERGE_RIGHT_FROM, merger_idx);
                                        if mtdt_idx + 1 == mergee_idx {
                                            layers.merge(SYM_EMPTY, merger_idx);
                                        } else {
                                            layers.merge(SYM_HORIZONTAL, merger_idx);
                                        }
                                        is_drawing = true;
                                    } else if is_drawing {
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
                            } else {
                                // After the commit
                                if is_merger_found && !is_merged_before {
                                    if mtdt.parents.len() == 1
                                        && metadata.parents.contains(mtdt.parents.first().unwrap())
                                    {
                                        layers.merge(SYM_MERGE_LEFT_FROM, merger_idx);
                                        layers.merge(SYM_EMPTY, merger_idx);
                                        is_drawing = false;
                                    } else if is_drawing {
                                        layers.merge(SYM_HORIZONTAL, merger_idx);
                                        layers.merge(SYM_HORIZONTAL, merger_idx);
                                    } else {
                                        layers.merge(SYM_EMPTY, merger_idx);
                                        layers.merge(SYM_EMPTY, merger_idx);
                                    }
                                }
                            }
                        }

                        if !is_merger_found {
                            // Count how many dummies in the end to get the real last element, append there
                            let mut idx = buffer.borrow().curr.len() - 1;
                            let mut trailing_dummies = 0;
                            for (i, c) in buffer.borrow().curr.iter().enumerate().rev() {
                                if !c.is_dummy() {
                                    idx = i;
                                    break;
                                } else {
                                    trailing_dummies += 1;
                                }
                            }

                            if trailing_dummies > 0
                                && buffer.borrow().prev.len() > idx
                                && buffer.borrow().prev[idx + 1].is_dummy()
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
                            merger_oid = Some(metadata.oid);
                        }
                    }
                } else {
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.commit(SYM_EMPTY, lane_idx);
                    if metadata.parents.contains(&head_oid) && lane_idx == 0 {
                        layers.pipe_custom(SYM_VERTICAL_DOTTED, lane_idx, COLOR_GREY_500);
                    } else {
                        layers.pipe(SYM_VERTICAL, lane_idx);
                    }
                    layers.pipe(SYM_EMPTY, lane_idx);
                }

                lane_idx += 1;
            }
            if !is_commit_found {
                if tips.contains_key(&oid) {
                    color.borrow_mut().alternate(lane_idx);
                    tip_colors.insert(oid, color.borrow().get(lane_idx));
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
        if let Some(sha) = merger_oid {
            buffer.borrow_mut().merger(sha);
        }
        buffer.borrow_mut().backup();

        // Serialize
        oids.push(oid);

        // Render
        render_graph(&oid, &mut lines_graph, spans_graph);
        render_branches(&oid, &mut lines_branches, &tips, &tip_colors, &commit);
        render_messages(&commit, &mut lines_messages);
        render_buffer(&buffer, &mut lines_buffer);
    }

    Walked {
        oids,
        tips,
        lines_graph,
        lines_branches,
        lines_messages,
        lines_buffer,
    }
}
