use std::{cell::RefCell, collections::HashMap};

use git2::{Commit, Oid, Time};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{core::buffer::Buffer, utils::colors::COLOR_TEXT};

pub fn render_graph(sha: &Oid, graph: &mut Vec<Line>, spans_graph: Vec<Span<'static>>) {
    let span_sha = Span::styled(sha.to_string()[..6].to_string(), COLOR_TEXT);
    let mut spans = Vec::new();
    spans.push(span_sha);
    spans.push(Span::raw(" ".to_string()));
    spans.extend(spans_graph);
    graph.push(Line::from(spans));
}

pub fn render_branches(
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
    branches.push(Line::from(spans));
}

pub fn render_messages(commit: &Commit<'_>, messages: &mut Vec<Line>) {
    let mut spans = Vec::new();
    let span_message = Span::styled(
        commit.summary().unwrap_or("<no message>").to_string(),
        Style::default().fg(COLOR_TEXT),
    );
    spans.push(span_message);
    messages.push(Line::from(spans));
}

pub fn render_buffer(
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

    // let formatted_buffer: String = _buffer.borrow().curr.iter().map(|metadata| {
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
