#[rustfmt::skip]
use std::{
    cell::Cell,
    collections::HashMap,
    io,
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::Rect,
    style::Color,
    text::{
        Line,
    },
};
#[rustfmt::skip]
use crate::{
    core::walker::walk,
};

#[derive(Default)]
pub struct Layout {
    pub title_left: Rect,
    pub title_right: Rect,
    pub graph: Rect,
    pub inspector: Rect,
    pub status_top: Rect,
    pub status_bottom: Rect,
    pub statusbar_left: Rect,
    pub statusbar_right: Rect,
}

#[derive(PartialEq, Eq)]
pub enum Panes {
    Graph,
    Inspector,
    StatusTop,
    StatusBottom
}

pub struct App {
    // General
    pub path: String,
    pub repo: Repository,

    // Data
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub tip_colors: HashMap<Oid, Color>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub oid_branch_map: HashMap<Oid, Vec<String>>,

    // Lines
    pub lines_graph: Vec<Line<'static>>,
    pub lines_branches: Vec<Line<'static>>,
    pub lines_messages: Vec<Line<'static>>,
    pub lines_buffers: Vec<Line<'static>>,

    // Interface
    pub layout: Layout,

    // Panes
    pub is_minimal: bool,
    pub is_status: bool,
    pub is_inspector: bool,
    pub is_modal: bool,
    pub focus: Panes,
    
    // Graph
    pub graph_selected: usize,
    pub graph_scroll: Cell<usize>,
    
    // Status top
    pub status_top_selected: usize,
    pub status_top_scroll: Cell<usize>,
    
    // Status bottom
    pub status_bottom_selected: usize,
    pub status_bottom_scroll: Cell<usize>,

    // Modal branch
    pub modal_selected: i32,

    // Exit
    pub is_exit: bool,    
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.reload();

        while !self.is_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }
    
    pub fn draw(&mut self, frame: &mut Frame) {
        self.layout(frame);
        self.draw_title(frame);
        self.draw_graph(frame);
        if self.is_status {self.draw_status(frame);}
        if self.is_inspector && self.graph_selected != 0 {self.draw_inspector(frame);}
        self.draw_statusbar(frame);
        self.draw_modal(frame);
    }

    pub fn reload(&mut self) {
        let walked = walk(&self.repo);
        self.oids = walked.oids;
        self.tips = walked.tips;
        self.tip_colors = walked.tip_colors;
        self.branch_oid_map = walked.branch_oid_map;
        self.oid_branch_map = walked.oid_branch_map;
        self.lines_graph = walked.lines_graph;
        self.lines_branches = walked.lines_branches;
        self.lines_messages = walked.lines_messages;
        self.lines_buffers = walked.lines_buffer;
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
