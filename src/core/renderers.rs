#[rustfmt::skip]
use im::Vector;
#[rustfmt::skip]
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use ratatui::{
    style::{
        Style,
        Color
    },
    text::{
        Line,
        Span
    },
};
#[rustfmt::skip]
use crate::{
    core::chunk::Chunk,
    git::queries::helpers::UncommittedChanges,
    helpers::colors::ColorPicker,
    layers,
};
#[rustfmt::skip]
use crate::{
    helpers::{
        palette::*,
        symbols::*
    }
};

pub fn render_graph_range(
    oids: &[Oid],
    tips: &HashMap<Oid, Vec<String>>,
    tip_colors: &mut HashMap<Oid, Color>,
    history: &Vector<Vector<Chunk>>,
    head_oid: Oid,
    start: usize,
    end: usize,
) -> Vec<Line<'static>> {
    // Clamp the range to valid indices
    let start = start.min(history.len());
    let end = end.min(history.len().saturating_sub(1));
    let mut layers = layers!(Rc::new(RefCell::new(ColorPicker::default())));
    let mut lines: Vec<Line> = Vec::new();
    let color = Rc::new(RefCell::new(ColorPicker::default()));

    // Go through the commits, inferring the graph
    for (global_idx, oid) in oids.iter().enumerate().take(end).skip(start) {
        layers.clear();
        let mut spans = vec![Span::raw(" ")];

        // Iterate over the buffer chunks, rendering the graph line
        let mut is_commit_found = false;
        let mut is_merged_before = false;
        let mut lane_idx = 0;

        let prev = history.get(global_idx);
        let last = history.get(global_idx + 1).unwrap();

        if *oid == Oid::zero() {
            lines.push(Line::from(Span::styled(
                " ◌",
                Style::default().fg(COLOR_GREY_400),
            )));
            continue;
        }

        for chunk in last.iter() {
            if chunk.is_dummy() {
                if let Some(prev_snapshot) = prev
                    && let Some(prev) = prev_snapshot.get(lane_idx)
                {
                    if (prev.parent_a.is_some() && prev.parent_b.is_none())
                        || (prev.parent_a.is_none() && prev.parent_b.is_some())
                    {
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
            } else if Some(oid) == chunk.oid.as_ref() {
                is_commit_found = true;
                let is_two_parents = chunk.parent_a.is_some() && chunk.parent_b.is_some();
                if is_two_parents && !tips.contains_key(oid) {
                    layers.commit(SYM_MERGE, lane_idx);
                } else if tips.contains_key(oid) {
                    layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                    tip_colors.insert(*oid, color.borrow().get(lane_idx));
                } else {
                    layers.commit(SYM_COMMIT, lane_idx);
                }
                layers.commit(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);

                // Check if commit is being merged into
                let mut is_mergee_found = false;
                let mut is_drawing = false;
                if is_two_parents {
                    let mut is_merger_found = false;
                    let mut merger_idx: usize = 0;

                    for chunk_nested in last {
                        if ((chunk_nested.parent_a.is_some() && chunk_nested.parent_b.is_none())
                            || (chunk_nested.parent_a.is_none() && chunk_nested.parent_b.is_some()))
                            && chunk.parent_b.as_ref() == chunk_nested.parent_a.as_ref()
                        {
                            is_merger_found = true;
                            break;
                        }
                        merger_idx += 1;
                    }

                    let mut mergee_idx: usize = 0;
                    for chunk_nested in last {
                        if Some(oid) == chunk_nested.oid.as_ref() {
                            break;
                        }
                        mergee_idx += 1;
                    }

                    for (chunk_nested_idx, chunk_nested) in last.iter().enumerate() {
                        if !is_mergee_found {
                            if Some(oid) == chunk_nested.oid.as_ref() {
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
                                } else if ((chunk_nested.parent_a.is_some()
                                    && chunk_nested.parent_b.is_none())
                                    || (chunk_nested.parent_a.is_none()
                                        && chunk_nested.parent_b.is_some()))
                                    && (chunk.parent_a.as_ref() == chunk_nested.parent_a.as_ref()
                                        || chunk.parent_b.as_ref()
                                            == chunk_nested.parent_a.as_ref())
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
                                if ((chunk_nested.parent_a.is_some()
                                    && chunk_nested.parent_b.is_none())
                                    || (chunk_nested.parent_a.is_none()
                                        && chunk_nested.parent_b.is_some()))
                                    && (chunk.parent_a.as_ref() == chunk_nested.parent_a.as_ref()
                                        || chunk.parent_b.as_ref()
                                            == chunk_nested.parent_a.as_ref())
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
                        let mut idx = last.len() - 1;
                        let mut trailing_dummies = 0;
                        for (i, c) in last.iter().enumerate().rev() {
                            if !c.is_dummy() {
                                idx = i;
                                break;
                            } else {
                                trailing_dummies += 1;
                            }
                        }

                        if trailing_dummies > 0
                            && prev.is_some()
                            && prev.unwrap().len() > idx
                            && prev.unwrap()[idx + 1].is_dummy()
                        {
                            layers.merge(SYM_BRANCH_DOWN, idx + 1);
                            layers.merge(SYM_EMPTY, idx + 1);
                        } else if trailing_dummies > 0 {
                            // Calculate how many lanes before we reach the branch character
                            for _ in lane_idx..idx {
                                layers.merge(SYM_HORIZONTAL, idx + 1);
                                layers.merge(SYM_HORIZONTAL, idx + 1);
                            }

                            layers.merge(SYM_MERGE_LEFT_FROM, idx + 1);
                            layers.merge(SYM_EMPTY, idx + 1);
                        } else {
                            // Calculate how many lanes before we reach the branch character
                            for _ in lane_idx..idx {
                                layers.merge(SYM_HORIZONTAL, idx + 1);
                                layers.merge(SYM_HORIZONTAL, idx + 1);
                            }

                            layers.merge(SYM_BRANCH_DOWN, idx + 1);
                            layers.merge(SYM_EMPTY, idx + 1);
                        }
                    }
                }
            } else {
                layers.commit(SYM_EMPTY, lane_idx);
                layers.commit(SYM_EMPTY, lane_idx);
                if (chunk.parent_a.as_ref() == Some(&head_oid)
                    || chunk.parent_b.as_ref() == Some(&head_oid))
                    && lane_idx == 0
                {
                    layers.pipe_custom(SYM_VERTICAL_DOTTED, lane_idx, COLOR_GREY_500);
                } else {
                    layers.pipe(SYM_VERTICAL, lane_idx);
                }
                layers.pipe(SYM_EMPTY, lane_idx);
            }

            lane_idx += 1;
        }

        if !is_commit_found {
            if tips.contains_key(oid) {
                layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                tip_colors.insert(*oid, color.borrow().get(lane_idx));
            } else {
                layers.commit(SYM_COMMIT, lane_idx);
            };
            layers.commit(SYM_EMPTY, lane_idx);
            layers.pipe(SYM_EMPTY, lane_idx);
            layers.pipe(SYM_EMPTY, lane_idx);
        }

        // Blend layers into the graph
        layers.bake(&mut spans);

        // Render
        lines.push(Line::from(spans));
    }

    lines
}

#[allow(dead_code)]
pub fn render_buffer_range(
    history: &Vector<Vector<Chunk>>,
    start: usize,
    end: usize,
) -> Vec<Line<'_>> {
    // Clamp the range to valid indices
    let start = start.min(history.len());
    let end = end.min(history.len());
    let mut lines_buffer: Vec<Line> = Vec::new();
    // Iterate over the selected snapshots
    for snapshot in history.iter().skip(start).take(end - start) {
        let mut spans = Vec::new();

        let formatted_snapshot: String = snapshot
            .iter()
            .map(|metadata| {
                let oid_str = metadata
                    .oid
                    .as_ref()
                    .map_or("--".to_string(), |o| o.to_string());

                let parents_formatted = match (&metadata.parent_a, &metadata.parent_b) {
                    (Some(a), Some(b)) => format!("{:.2},{:.2}", a, b),
                    (Some(a), None) => format!("{:.2},--", a),
                    (None, Some(b)) => format!("--,{:.2}", b),
                    (None, None) => "--,--".to_string(),
                };

                format!("{}({:<5})", &oid_str[..2], parents_formatted)
            })
            .collect::<Vec<String>>()
            .join(" ");

        spans.push(Span::styled(
            formatted_snapshot,
            Style::default().fg(COLOR_TEXT),
        ));
        lines_buffer.push(Line::from(spans));
    }

    lines_buffer
}

#[allow(clippy::too_many_arguments)]
pub fn render_message_range(
    repo: &Repository,
    oids: &[Oid],
    tips: &HashMap<Oid, Vec<String>>,
    tip_colors: &mut HashMap<Oid, Color>,
    history: &Vector<Vector<Chunk>>,
    start: usize,
    end: usize,
    selected: usize,
    uncommitted: &UncommittedChanges,
) -> Vec<Line<'static>> {
    // Clamp the range to valid indices
    let start = start.min(history.len());
    let end = end.min(history.len().saturating_sub(1));
    let mut lines: Vec<Line> = Vec::new();

    // Go through the commits, inferring the graph
    for global_idx in start..end {
        let oid = *oids.get(global_idx).unwrap();
        let mut spans = Vec::new();

        if oid != Oid::zero() {
            let commit = repo.find_commit(*oids.get(global_idx).unwrap()).unwrap();

            if let Some(branches) = tips.get(&oid) {
                for branch in branches {
                    spans.push(Span::styled(
                        format!("{} {} ", SYM_COMMIT_BRANCH, branch),
                        Style::default().fg(if let Some(color) = tip_colors.get(&oid) {
                            *color
                        } else {
                            COLOR_TEXT
                        }),
                    ));
                }
            }

            spans.push(Span::styled(
                commit.summary().unwrap_or("⊘ no message").to_string(),
                Style::default().fg(if global_idx == selected {
                    COLOR_GREY_400
                } else {
                    COLOR_TEXT
                }),
            ));

            lines.push(Line::from(spans));
        } else {
            let color = if global_idx == selected {
                COLOR_GREY_400
            } else {
                COLOR_GREY_600
            };
            if uncommitted.modified_count > 0 {
                spans.push(Span::styled("~ ", Style::default().fg(COLOR_BLUE)));
                spans.push(Span::styled(
                    format!("{} ", uncommitted.modified_count),
                    Style::default().fg(color),
                ));
            }
            if uncommitted.added_count > 0 {
                spans.push(Span::styled("+ ", Style::default().fg(COLOR_GREEN)));
                spans.push(Span::styled(
                    format!("{} ", uncommitted.added_count),
                    Style::default().fg(color),
                ));
            }
            if uncommitted.deleted_count > 0 {
                spans.push(Span::styled("- ", Style::default().fg(COLOR_RED)));
                spans.push(Span::styled(
                    format!("{} ", uncommitted.deleted_count),
                    Style::default().fg(color),
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    lines
}
