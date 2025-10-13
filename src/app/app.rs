#[rustfmt::skip]
use std::{
    cell::{
        Cell,
        RefCell
    },
    sync::{
        Arc,
        mpsc::{
            channel,
        }
    },
    collections::{
        HashMap,
        HashSet
    },
    time::{
        Duration
    },
    thread,
    io,
};
#[rustfmt::skip]
use crossterm::event::poll;
#[rustfmt::skip]
use edtui::{
    EditorEventHandler,
    EditorState
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
    widgets::{
        ListItem
    },
    text::{
        Line,
        Span
    },
};
use crate::core::walker::{WalkContext, WalkContextOutput};
#[rustfmt::skip]
use crate::{
    layers,
    core::{
        layers::{
            LayersContext,
        },
        walker::{
            LazyWalker
        },
        buffer::{
            Buffer
        },
    },
    helpers::{
        colors::{
            ColorPicker
        }
    },
    git::{
        queries::{
            diffs::{
                get_filenames_diff_at_workdir
            },
            commits::{
                get_tip_oids
            },
            helpers::{
                FileChange,
                UncommittedChanges
            }
        }
    },
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
pub enum Viewport {
    Graph,
    Viewer,
    Editor
}

#[derive(PartialEq, Eq)]
pub enum Focus {
    Viewport,
    Inspector,
    StatusTop,
    StatusBottom,
    ModalActions,
    ModalCheckout,
    ModalCommit,
}

pub struct App {
    // General
    pub logo: Vec<Span<'static>>,
    pub path: String,
    pub repo: Arc<Repository>,
    pub walker: LazyWalker,
    pub log: Vec<String>,
    
    // User
    pub name: String,
    pub email: String,

    // Walker utilities    
    pub color: Arc<RefCell<ColorPicker>>,
    pub buffer: RefCell<Buffer>,
    pub layers: LayersContext,
    pub walker_rx: Option<std::sync::mpsc::Receiver<WalkContextOutput>>,

    // Walker data
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub oid_colors: HashMap<Oid, Color>,
    pub tip_colors: HashMap<Oid, Color>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub oid_branch_map: HashMap<Oid, HashSet<String>>,
    pub uncommitted: UncommittedChanges,

    // Walker lines
    pub lines_graph: Vec<Line<'static>>,
    pub lines_branches: Vec<Line<'static>>,
    pub lines_messages: Vec<Line<'static>>,
    pub lines_buffers: Vec<Line<'static>>,

    // Cache
    pub current_diff: Vec<FileChange>,
    pub file_name: Option<String>,
    pub viewer_lines: Vec<ListItem<'static>>,

    // Interface
    pub layout: Layout,

    // Focus
    pub is_minimal: bool,
    pub is_status: bool,
    pub is_inspector: bool,
    pub viewport: Viewport,
    pub focus: Focus,
    
    // Graph
    pub graph_selected: usize,
    pub graph_scroll: Cell<usize>,
    
    // Viewer
    pub viewer_selected: usize,
    pub viewer_scroll: Cell<usize>,

    // Editor
    pub file_editor: EditorState,
    pub file_editor_event_handler: EditorEventHandler,
    
    // Inspector
    pub inspector_selected: usize,
    pub inspector_scroll: Cell<usize>,
    
    // Status top
    pub status_top_selected: usize,
    pub status_top_scroll: Cell<usize>,
    
    // Status bottom
    pub status_bottom_selected: usize,
    pub status_bottom_scroll: Cell<usize>,

    // Modal checkout
    pub modal_checkout_selected: i32,

    // Modal commit
    pub commit_editor: EditorState,
    pub commit_editor_event_handler: EditorEventHandler,

    // Exit
    pub is_exit: bool,    
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {

        self.reload();

        // Main loop
        while !self.is_exit {

            // Check if the background walk is done
            if let Some(rx) = &self.walker_rx {
                if let Ok(result) = rx.try_recv() {
                    self.oids = result.oids;
                    self.tips = result.tips;
                    self.oid_colors = result.oid_colors;
                    self.tip_colors = result.tip_colors;
                    self.branch_oid_map = result.branch_oid_map;
                    self.oid_branch_map = result.oid_branch_map;
                    self.uncommitted = result.uncommitted;
                    self.lines_graph = result.lines_graph;
                    self.lines_branches = result.lines_branches;
                    self.lines_messages = result.lines_messages;
                    self.lines_buffers = result.lines_buffers;

                    if !result.again {
                        self.walker_rx = None;
                    }
                }
            }

            // Draw the user interface
            terminal.draw(|frame| self.draw(frame))?;

            // Poll for events with a timeout
            if poll(Duration::from_millis(100))? {
                // Handle events
                self.handle_events()?;
            }
            
            std::thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    }
    
    pub fn draw(&mut self, frame: &mut Frame) {
        // Compute the layout
        self.layout(frame);

        // Viewport
        match self.viewport {
            Viewport::Graph => {
                self.draw_graph(frame);
            }
            Viewport::Viewer => {
                self.draw_viewer(frame);
            }
            Viewport::Editor => {
                self.draw_editor(frame);
            }
            _ => {}
        }

        // Main layout
        self.draw_title(frame);
        if self.is_status {self.draw_status(frame);}
        if self.is_inspector && self.graph_selected != 0 {self.draw_inspector(frame);}
        self.draw_statusbar(frame);

        // Modals
        match self.focus {
            Focus::ModalActions => {
                self.draw_modal_actions(frame);
            }
            Focus::ModalCheckout => {
                self.draw_modal_checkout(frame);
            }
            Focus::ModalCommit => {
                self.draw_modal_commit(frame);
            }
            _ => {}
        }
    }

    pub fn reload(&mut self) {

        // Reset the walker
        self.walker.reset(self.repo.clone()).expect("Failed to reset walker");

        // Reset utilities
        self.color = Arc::new(RefCell::new(ColorPicker::default()));
        self.buffer = RefCell::new(Buffer::default());
        self.layers = layers!(self.color.clone());

        // Topologically sorted list of oids including the uncommited, for the sake of order
        self.oids = vec![Oid::zero()];
        // Mapping of tip oids of the branches to the branch names
        self.tips = get_tip_oids(&self.repo);
        // Mapping of oids to lanes
        self.oid_colors = HashMap::new();
        // Mapping of tip oids of the branches to the colors
        self.tip_colors = HashMap::new();
        // Mapping of every oid to every branch it is a part of
        self.oid_branch_map = HashMap::new();
        self.branch_oid_map = HashMap::new();
        // Get uncomitted changes info
        self.uncommitted = get_filenames_diff_at_workdir(&self.repo).expect("Error");
        // Walker lines
        self.lines_graph = Vec::new();
        self.lines_branches = Vec::new();
        self.lines_messages = Vec::new();
        self.lines_buffers = Vec::new();

        // First walk
        self.walk();
    }

    pub fn walk(&mut self) {

        // Create a channel
        let (tx, rx) = channel();
        self.walker_rx = Some(rx);

        let path = self.path.clone();
        let color = (*self.color.borrow()).clone();
        
        // Spawn a thread that computes something
        thread::spawn(move || {
            let repo = Arc::new(Repository::open(path).expect("Failed to open repo"));
            let walker =  LazyWalker::new(repo.clone()).expect("Error");
            let color =  Arc::new(RefCell::new(color));
            let buffer =  RefCell::new(Buffer::default());
            let layers =  layers!(Arc::new(RefCell::new(ColorPicker::default())));
            let oids = vec![Oid::zero()];
            let tips = get_tip_oids(&repo);
            let oid_colors = HashMap::new();
            let tip_colors = HashMap::new();
            let oid_branch_map = HashMap::new();
            let branch_oid_map = HashMap::new();
            let uncommitted = get_filenames_diff_at_workdir(&repo).expect("Error");

            let mut walk_ctx = WalkContext {
                repo,
                walker,
                color,
                buffer,
                layers,
                oids,
                tips,
                oid_colors,
                tip_colors,
                oid_branch_map,
                branch_oid_map,
                uncommitted,
                lines_graph: Vec::new(),
                lines_branches: Vec::new(),
                lines_messages: Vec::new(),
                lines_buffers: Vec::new(),
            };
            
            loop {

                let again = walk_ctx.walk(10000);

                tx.send(WalkContextOutput {
                    oids: walk_ctx.oids.clone(),
                    tips: walk_ctx.tips.clone(),
                    oid_colors: walk_ctx.oid_colors.clone(),
                    tip_colors: walk_ctx.tip_colors.clone(),
                    oid_branch_map: walk_ctx.oid_branch_map.clone(),
                    branch_oid_map: walk_ctx.branch_oid_map.clone(),
                    uncommitted: walk_ctx.uncommitted.clone(),
                    lines_graph: walk_ctx.lines_graph.clone(),
                    lines_branches: walk_ctx.lines_branches.clone(),
                    lines_messages: walk_ctx.lines_messages.clone(),
                    lines_buffers: walk_ctx.lines_buffers.clone(),
                    again
                }).expect("Error");

                if !again { break }
            }
            
        });
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
