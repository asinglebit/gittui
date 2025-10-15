#[rustfmt::skip]
use std::{
    cell::{
        Cell,
        RefCell
    },
    rc::Rc,
    sync::{
        mpsc::{
            channel
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
use edtui::{
    EditorEventHandler,
    EditorState
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
use ratatui::{style::Style, widgets::{Block, Borders}};
#[rustfmt::skip]
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::Rect,
    style::Color,
    crossterm::event,
    widgets::{
        ListItem
    },
    text::{
        Line,
        Span
    },
};
use crate::helpers::palette::COLOR_BORDER;
#[rustfmt::skip]
use crate::{
    layers,
    core::{
        layers::{
            LayersContext,
        },
        walker::{
            LazyWalker,
            Walker,
            WalkerOutput
        },
        buffer::{
            Buffer
        },
    },
    helpers::{
        colors::{
            ColorPicker
        },
        spinner::{
            Spinner
        }
    },
    git::{
        queries::{
            diffs::{
                get_filenames_diff_at_workdir
            },
            commits::{
                get_tip_oids,
                get_git_user_info
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
    pub app: Rect,
    pub branches: Rect,
    pub branches_scrollbar: Rect,
    pub graph: Rect,
    pub graph_scrollbar: Rect,
    pub inspector: Rect,
    pub inspector_scrollbar: Rect,
    pub status_top: Rect,
    pub status_top_scrollbar: Rect,
    pub status_bottom: Rect,
    pub status_bottom_scrollbar: Rect,
    pub statusbar_left: Rect,
    pub statusbar_right: Rect,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Viewport {
    Graph,
    Viewer,
    Editor,
    Settings
}

#[derive(PartialEq, Eq)]
pub enum Focus {
    Viewport,
    Inspector,
    StatusTop,
    StatusBottom,
    Branches,
    ModalActions,
    ModalCheckout,
    ModalCommit,
}

pub struct App {
    // General
    pub logo: Vec<Span<'static>>,
    pub path: String,
    pub repo: Rc<Repository>,
    pub hint: String,
    pub spinner: Spinner,

    // User
    pub name: String,
    pub email: String,

    // Walker utilities
    pub color: Rc<RefCell<ColorPicker>>,
    pub buffer: RefCell<Buffer>,
    pub layers: LayersContext,
    pub walker_rx: Option<std::sync::mpsc::Receiver<WalkerOutput>>,

    // Walker data
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub oid_colors: HashMap<Oid, Color>,
    pub tip_colors: HashMap<Oid, Color>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub oid_branch_map: HashMap<Oid, HashSet<String>>,
    pub uncommitted: UncommittedChanges,

    // Cache
    pub current_diff: Vec<FileChange>,
    pub file_name: Option<String>,
    pub viewer_lines: Vec<ListItem<'static>>,
    pub oid_branch_vec: Vec<(Oid, String)>,
    pub visible_branch_oids: HashSet<Oid>,

    // Interface
    pub layout: Layout,

    // Focus
    pub is_minimal: bool,
    pub is_branches: bool,
    pub is_status: bool,
    pub is_inspector: bool,
    pub viewport: Viewport,
    pub focus: Focus,

    // Branches
    pub branches_selected: usize,
    pub branches_scroll: Cell<usize>,

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

impl App  {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
    
        self.reload();

        // Main loop
        while !self.is_exit {

            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                    self.handle_events()?;
            }

            // Check if the background walk is done
            if let Some(rx) = &self.walker_rx
                && let Ok(result) = rx.try_recv() {
                    self.oids = result.oids;
                    self.tips = result.tips;
                    self.branch_oid_map = result.branch_oid_map;
                    self.uncommitted = result.uncommitted;
                    self.buffer = result.buffer;
                    self.oid_branch_vec = self.tips.iter().flat_map(|(oid, branches)| {
                        branches.iter().map(move |branch| (*oid, branch.clone()))
                    }).collect();

                    if !result.again {
                        // self.walker_rx = None;
                        self.spinner.stop();
                    }
                }

            // Draw the user interface
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        // Compute the layout
        self.layout(frame);

        frame.render_widget( Block::default()
            // .title(vec![
            //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
            //     Span::styled(" graph ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
            //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
            // ])
            // .title_alignment(ratatui::layout::Alignment::Right)
            // .title_style(Style::default().fg(COLOR_GREY_400))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COLOR_BORDER))
            .border_type(ratatui::widgets::BorderType::Rounded), self.layout.app);
                

        // Main layout
        self.draw_title(frame);

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
            Viewport::Settings => {
                self.draw_settings(frame);
            }
        }

        // Panes
        match self.viewport {
            Viewport::Settings => {}
            _ => {
                if self.is_branches {
                    self.draw_branches(frame);
                }
                if self.is_status {
                    self.draw_status(frame);
                }
                if self.is_inspector && self.graph_selected != 0 {
                    self.draw_inspector(frame);
                }
            }
        }

        // Status bar
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
        if self.spinner.is_running() { return; }

        // Get user credentials
        let (name, email) = get_git_user_info(&self.repo).expect("Error");
        self.name = name.unwrap();
        self.email = email.unwrap();

        // Reset utilities
        self.color = Rc::new(RefCell::new(ColorPicker::default()));
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
        // Restart the spinner
        self.spinner.start();
        // First walk
        self.walk();
    }

    pub fn walk(&mut self) {
        // Create a channel
        let (tx, rx) = channel();
        self.walker_rx = Some(rx);

        // Copy the repo path
        let path = self.path.clone();
        let visible_branch_oids = self.visible_branch_oids.clone();

        // Spawn a thread that computes something
        thread::spawn(move || {
            // Create the walker
            let mut walk_ctx = Walker::new(path, 10000, visible_branch_oids).expect("Error");

            // Pagination loop
            loop {
                // Parse a chunk
                let again = walk_ctx.walk();

                // Send the message to the main thread
                tx.send(WalkerOutput {
                    oids: walk_ctx.oids.clone(),
                    tips: walk_ctx.tips.clone(),
                    branch_oid_map: walk_ctx.branch_oid_map.clone(),
                    uncommitted: walk_ctx.uncommitted.clone(),
                    buffer: walk_ctx.buffer.clone(),
                    again,
                })
                .expect("Error");

                // Break the loop
                if !again {
                    break;
                }
            }
        });
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
