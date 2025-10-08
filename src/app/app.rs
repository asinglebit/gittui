#[rustfmt::skip]
use std::{
    cell::Cell,
    collections::HashMap,
    io,
};
#[rustfmt::skip]
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use ratatui::{
    DefaultTerminal,
    text::{
        Line,
    },
};
#[rustfmt::skip]
use crate::{
    core::walker::walk,
};

pub struct App {
    // General
    pub path: String,
    pub repo: Repository,

    // Data
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,

    // Lines
    pub lines_graph: Vec<Line<'static>>,
    pub lines_branches: Vec<Line<'static>>,
    pub lines_messages: Vec<Line<'static>>,
    pub lines_buffers: Vec<Line<'static>>,

    // Interface
    pub scroll: Cell<usize>,
    pub files_scroll: Cell<usize>,
    pub selected: usize,
    pub is_modal: bool,
    pub is_minimal: bool,
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

    pub fn reload(&mut self) {
        let walked = walk(&self.repo);
        self.oids = walked.oids;
        self.tips = walked.tips;
        self.lines_graph = walked.lines_graph;
        self.lines_branches = walked.lines_branches;
        self.lines_messages = walked.lines_messages;
        self.lines_buffers = walked.lines_buffer;
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
