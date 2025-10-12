#[rustfmt::skip]
use std::{
    cell::RefCell,
    collections::HashMap
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use ratatui::{
    style::{
        Color,
    },
    text::{
        Line,
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
            render_messages,
            render_uncommitted
        },
    },
    git::queries::{
        commits::{
            get_branch_oids,
            get_sorted_oids,
            // get_timestamps,
            get_tip_oids,
        },
        diffs::{
            get_filenames_diff_at_workdir,
            UncommittedChanges
        }
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
    pub oid_colors: HashMap<Oid, Color>,
    pub tip_colors: HashMap<Oid, Color>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub oid_branch_map: HashMap<Oid, Vec<String>>,
    pub uncommitted: UncommittedChanges,
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
    let mut oids = vec![Oid::zero()];
    // Mapping of tip oids of the branches to the branch names
    let tips: HashMap<Oid, Vec<String>> = get_tip_oids(repo);
    // Mapping of oids to lanes
    let mut oid_colors: HashMap<Oid, Color> = HashMap::new();
    // Mapping of tip oids of the branches to the colors
    let mut tip_colors: HashMap<Oid, Color> = HashMap::new();
    // Mapping of every oid to every branch it is a part of
    let (oid_branch_map, branch_oid_map) = get_branch_oids(repo, &tips);
    // Timestamps of every oid
    // let timestamps: HashMap<Oid, (Time, Time, Time)> = get_timestamps(repo, &branches);
    // Topologically sorted list of oids including the uncommited
    let sorted: Vec<Oid> = get_sorted_oids(repo);
    // Get uncomitted changes info
    let uncommitted = get_filenames_diff_at_workdir(repo).expect("Error");
    // Get current head oid
    let head_oid = repo.head().unwrap().target().unwrap();

    // Make a fake commit for unstaged changes
    render_uncommitted(
        head_oid,
        &uncommitted,
        &mut lines_graph,
        &mut lines_branches,
        &mut lines_messages,
        &mut lines_buffer,
    );
    buffer
        .borrow_mut()
        .update(Chunk::uncommitted(vec![head_oid]));

    // Go through the commits, inferring the graph
    for oid in sorted {
        let mut merger_oid = None;

        layers.clear();
        let commit = repo.find_commit(oid).unwrap();
        let parents: Vec<Oid> = commit.parent_ids().collect();
        let chunk = Chunk::commit(oid, parents);

        let mut spans_graph = Vec::new();

        // Update
        buffer.borrow_mut().update(chunk);

        {
            // Otherwise (meaning we reached a tip, merge or a non-branching commit)
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for chunk in &buffer.borrow().curr {
                if chunk.is_dummy() {
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
                } else if oid == chunk.oid {
                    is_commit_found = true;
                    oid_colors
                        .entry(oid)
                        .or_insert(color.borrow().get(lane_idx));

                    if chunk.parents.len() > 1 && !tips.contains_key(&oid) {
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
                    if chunk.parents.len() > 1 {
                        let mut is_merger_found = false;
                        let mut merger_idx: usize = 0;
                        for chunk_nested in &buffer.borrow().curr {
                            if chunk_nested.parents.len() == 1
                                && chunk.parents.last().unwrap()
                                    == chunk_nested.parents.first().unwrap()
                            {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for chunk_nested in &buffer.borrow().curr {
                            if oid == chunk_nested.oid {
                                break;
                            }
                            mergee_idx += 1;
                        }

                        for (chunk_nested_idx, chunk_nested) in
                            buffer.borrow().curr.iter().enumerate()
                        {
                            if !is_mergee_found {
                                if oid == chunk_nested.oid {
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
                                    } else if chunk_nested.parents.len() == 1
                                        && chunk
                                            .parents
                                            .contains(chunk_nested.parents.first().unwrap())
                                    {
                                        layers.merge(SYM_MERGE_RIGHT_FROM, merger_idx);
                                        if chunk_nested_idx + 1 == mergee_idx {
                                            layers.merge(SYM_EMPTY, merger_idx);
                                        } else {
                                            layers.merge(SYM_HORIZONTAL, merger_idx);
                                        }
                                        is_drawing = true;
                                    } else if is_drawing {
                                        if chunk_nested_idx + 1 == mergee_idx {
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
                                    if chunk_nested.parents.len() == 1
                                        && chunk
                                            .parents
                                            .contains(chunk_nested.parents.first().unwrap())
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
                            merger_oid = Some(chunk.oid);
                        }
                    }
                } else {
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.commit(SYM_EMPTY, lane_idx);
                    if chunk.parents.contains(&head_oid) && lane_idx == 0 {
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
        oid_colors,
        tip_colors,
        branch_oid_map,
        oid_branch_map,
        uncommitted,
        lines_graph,
        lines_branches,
        lines_messages,
        lines_buffer,
    }
}
