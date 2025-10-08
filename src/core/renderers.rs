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
    utils::{
        colors::{
            COLOR_TEXT,
            COLOR_GREY_400
        },
        symbols::SYM_UNCOMMITED
    }
};

pub fn render_uncommitted(
    head_oid: Oid,
    (new_count, modified_count, deleted_count): &(usize, usize, usize),
    lines_graph: &mut Vec<Line>,
    lines_branches: &mut Vec<Line>,
    lines_messages: &mut Vec<Line>,
    lines_buffer: &mut Vec<Line>,
) {
    let mut uncommited_line_spans = vec![Span::styled(
        format!("{} ", SYM_UNCOMMITED),
        Style::default().fg(COLOR_GREY_400),
    )];
    if *modified_count > 0 {
        uncommited_line_spans.push(Span::styled(
            format!("~{} ", modified_count),
            Style::default().fg(COLOR_GREY_400),
        ));
    }
    if *new_count > 0 {
        uncommited_line_spans.push(Span::styled(
            format!("+{} ", new_count),
            Style::default().fg(COLOR_GREY_400),
        ));
    }
    if *new_count > 0 {
        uncommited_line_spans.push(Span::styled(
            format!("-{} ", deleted_count),
            Style::default().fg(COLOR_GREY_400),
        ));
    }
    lines_branches.push(Line::from(uncommited_line_spans));
    lines_messages.push(Line::from(""));
    lines_buffer.push(Line::from(format!("UU({:.2},--)", head_oid)));
    lines_graph.push(Line::from(vec![
        Span::styled("······ ", Style::default().fg(COLOR_TEXT)),
        Span::styled(SYM_UNCOMMITED, Style::default().fg(COLOR_GREY_400)),
    ]));
}

pub fn render_graph(oid: &Oid, graph: &mut Vec<Line>, spans_graph: Vec<Span<'static>>) {
    let span_oid = Span::styled(oid.to_string()[..6].to_string(), COLOR_TEXT);
    let mut spans = Vec::new();
    spans.push(span_oid);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

pub fn render_branches(
    oid: &Oid,
    lines_branches: &mut Vec<Line>,
    tips: &HashMap<Oid, Vec<String>>,
    tip_colors: &HashMap<Oid, Color>,
    commit: &Commit<'_>,
) {
    let mut spans = Vec::new();
    let span_tips: Vec<Span<'_>> = tips
        .get(oid)
        .map(|branches| {
            branches
                .iter()
                .flat_map(|branch| {
                    vec![Span::styled(
                        format!("● {} ", branch),
                        Style::default().fg(*tip_colors.get(oid).unwrap_or(&Color::White)),
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
    lines_branches.push(Line::from(spans));
}

pub fn render_messages(commit: &Commit<'_>, lines_messages: &mut Vec<Line>) {
    let mut spans = Vec::new();
    let span_message = Span::styled(
        commit.summary().unwrap_or("<no message>").to_string(),
        Style::default().fg(COLOR_TEXT),
    );
    spans.push(span_message);
    lines_messages.push(Line::from(spans));
}

#[allow(dead_code)]
pub fn render_timestamps(
    oid: &Oid,
    timestamps: &HashMap<Oid, (Time, Time, Time)>,
    lines_timestamps: &mut Vec<Line>,
) {
    let mut spans = Vec::new();
    let time = timestamps.get(oid).unwrap().0.seconds();
    let o_time = timestamps.get(oid).unwrap().0.offset_minutes();
    let committer_time = timestamps.get(oid).unwrap().1.seconds();
    let o_committer_time = timestamps.get(oid).unwrap().1.offset_minutes();
    let author_time = timestamps.get(oid).unwrap().1.seconds();
    let o_author_time = timestamps.get(oid).unwrap().1.offset_minutes();
    let span_timestamp = Span::styled(
        format!(
            "{}:{:.3}:{}:{:.3}:{}:{:.3} ",
            time, o_time, committer_time, o_committer_time, author_time, o_author_time
        ),
        Style::default().fg(Color::DarkGray),
    );
    spans.push(span_timestamp);
    lines_timestamps.push(Line::from(spans));
}

pub fn render_buffer(buffer: &RefCell<Buffer>, lines_buffer: &mut Vec<Line>) {
    let mut spans = Vec::new();
    let formatted_buffer: String = buffer
        .borrow()
        .curr
        .iter()
        .map(|metadata| {
            format!(
                "{:.2}({:<5})",
                metadata.oid,
                if !metadata.parents.is_empty() {
                    let a = metadata
                        .parents
                        .iter()
                        .map(|oid| format!("{:.2}", oid))
                        .collect::<Vec<String>>();
                    let mut s = a.join(",");
                    if a.len() == 1 {
                        s.push(',');
                        s.push('-');
                        s.push('-');
                    }
                    s
                } else {
                    "--,--".to_string()
                },
            )
        })
        .collect::<Vec<String>>()
        .join(" ");
    let span_buffer = Span::styled(formatted_buffer, Style::default().fg(COLOR_TEXT));
    spans.push(span_buffer);
    lines_buffer.push(Line::from(spans));
}
