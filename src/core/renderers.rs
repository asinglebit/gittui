#[rustfmt::skip]
use std::{
    cell::RefCell,
    collections::HashMap
};
#[rustfmt::skip]
use git2::{
    Commit,
    Oid,
    Time
};
use im::Vector;
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
use crate::core::chunk::Chunk;
#[rustfmt::skip]
use crate::{
    core::buffer::Buffer,
    git::{
        queries::{
            helpers::{
                UncommittedChanges
            }
        }
    },
    helpers::{
        palette::*,
        symbols::*
    }
};

pub fn render_uncommitted(
    head_oid: Oid,
    lines_graph: &mut Vec<Line>,
) {
    lines_graph.push(Line::from(vec![
        Span::styled(" ", Style::default().fg(COLOR_TEXT)),
        Span::styled(SYM_UNCOMMITED, Style::default().fg(COLOR_GREY_400)),
    ]));
}

pub fn render_graph(oid: &Oid, graph: &mut Vec<Line>, spans_graph: Vec<Span<'static>>) {
    // let span_oid = Span::styled(oid.to_string()[..6].to_string(), COLOR_TEXT);
    let mut spans = Vec::new();
    // spans.push(span_oid);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

pub fn render_graph_range(
    history: &Vector<Vector<Chunk>>,
    start: usize,
    end: usize, // exclusive
) -> Vec<Line> {

    // Clamp the range to valid indices
    let start = start.min(history.len());
    let end = end.min(history.len());

    if start >= end {
        return Vec::new(); // nothing to render
    }

    let mut lines_graph: Vec<Line> = Vec::new();

    // // Iterate over the selected snapshots
    // for snapshot in history.iter().skip(start).take(end - start) {
    //     let mut spans = Vec::new();


    //     lines_graph.push(Line::from(spans));
    // }

    lines_graph
}


pub fn render_buffer_range(
    history: &Vector<Vector<Chunk>>,
    start: usize,
    end: usize, // exclusive
) -> Vec<Line> {

    // Clamp the range to valid indices
    let start = start.min(history.len());
    let end = end.min(history.len());

    if start >= end {
        return Vec::new(); // nothing to render
    }

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

                format!(
                    "{}({:<5})",
                    &oid_str[..2],
                    parents_formatted
                )
            })
            .collect::<Vec<String>>()
            .join(" ");

        spans.push(Span::styled(formatted_snapshot, Style::default().fg(COLOR_TEXT)));
        lines_buffer.push(Line::from(spans));
    }

    lines_buffer
}
