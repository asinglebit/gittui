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
    uncommitted: &UncommittedChanges,
    lines_graph: &mut Vec<Line>,
    lines_branches: &mut Vec<Line>,
    lines_messages: &mut Vec<Line>,
    lines_buffer: &mut Vec<Line>,
) {
    lines_messages.push(Line::from(""));
    lines_buffer.push(Line::from(format!("UU({:.2},--)", head_oid)));
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

pub fn render_buffer(buffer: &RefCell<Buffer>, lines_buffer: &mut Vec<Line>) {
    let mut spans = Vec::new();

    let formatted_buffer: String = buffer
        .borrow()
        .curr
        .iter()
        .map(|metadata| {
            let oid_str = metadata.oid.as_ref().map_or("--".to_string(), |o| o.to_string());

            let parents_formatted = match (&metadata.parent_a, &metadata.parent_b) {
                (Some(a), Some(b)) => format!("{:.2},{:.2}", a, b),
                (Some(a), None) => format!("{:.2},--", a),
                (None, Some(b)) => format!("--,{:.2}", b),
                (None, None) => "--,--".to_string(),
            };

            format!(
                "{}({:<5})",
                &oid_str[..2], // handle Option<Oid>
                parents_formatted
            )
        })
        .collect::<Vec<String>>()
        .join(" ");

    let span_buffer = Span::styled(formatted_buffer, Style::default().fg(COLOR_TEXT));
    spans.push(span_buffer);
    lines_buffer.push(Line::from(spans));
}