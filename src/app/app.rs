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
        },
        Arc,
        atomic::{
            AtomicBool
        }
    },
    collections::{
        HashMap
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
#[rustfmt::skip]
use indexmap::{
    IndexMap
};
#[rustfmt::skip]
use ratatui::{
    DefaultTerminal,
    Frame,
    layout::Rect,
    style::{
        Color,
        Style
    },
    crossterm::event,
    widgets::{
        ListItem,
        Block,
        Borders
    },
    text::{
        Span
    },
};
#[rustfmt::skip]
use crate::{
    layers,
    app::app_input::{
        Command,
        KeyBinding
    },
    core::{
        chunk::NONE,
        layers::{
            LayersContext,
        },
        walker::{
            Walker,
            WalkerOutput
        },
        buffer::{
            Buffer
        },
    },
    helpers::{
        palette::*,
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
    Splash,
    Settings
}

#[derive(PartialEq, Eq)]
pub enum Focus {
    Viewport,
    Inspector,
    StatusTop,
    StatusBottom,
    Branches,
    ModalCheckout,
    ModalSolo,
    ModalCommit,
    ModalCreateBranch,
    ModalDeleteBranch
}

#[derive(PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
}

#[derive(Clone)]
pub struct OidManager {
    pub zero: Oid,
    pub oids: Vec<Oid>,
    pub aliases: HashMap<Oid, u32>,
    pub sorted_aliases: Vec<u32>,
}

impl Default for OidManager {
    fn default() -> Self {
        OidManager {
            zero: Oid::zero(),
            oids: Vec::new(),
            aliases: HashMap::new(),
            sorted_aliases: vec![NONE],
        }
    }
}

impl OidManager {
    pub fn get_alias_by_oid(&mut self, oid: Oid) -> u32 {
        *self.aliases.entry(oid).or_insert_with(|| {
            self.oids.push(oid);
            self.oids.len() as u32 - 1
        })
    }

    pub fn get_alias_by_idx(&self, idx: usize) -> u32 {
        *self.sorted_aliases.get(idx).unwrap()
    }

    pub fn get_oid_by_alias(&self, alias: u32) -> &Oid {
        self.oids.get(alias as usize).unwrap_or(&self.zero)
    }

    pub fn get_oid_by_idx(&self, idx: usize) -> &Oid {
        let alias = *self.sorted_aliases.get(idx).unwrap_or(&NONE);
        self.oids.get(alias as usize).unwrap_or(&self.zero)
    }

    pub fn get_sorted_aliases(&self) -> &Vec<u32> {
        &self.sorted_aliases
    }

    pub fn append_sorted_alias(&mut self, alias: u32) {
        self.sorted_aliases.push(alias);
    }

    pub fn get_commit_count(&self) -> usize {
        self.sorted_aliases.len()
    }

    pub fn is_zero(&self, oid: &Oid) -> bool {
        self.zero == *oid
    }
}

#[derive(Default)]
pub struct BranchManager {
    pub local: HashMap<u32, Vec<String>>,
    pub remote: HashMap<u32, Vec<String>>,
    pub all: HashMap<u32, Vec<String>>,
    pub colors: HashMap<u32, Color>,
    pub sorted: Vec<(u32, String)>,
    pub indices: Vec<usize>,
    pub visible: HashMap<u32, Vec<String>>,
}

impl BranchManager {
    pub fn feed(
        &mut self,
        oid_manager: &OidManager,
        color: &Rc<RefCell<ColorPicker>>,
        lanes: &HashMap<u32, usize>,
        local: HashMap<u32, Vec<String>>,
        remote: HashMap<u32, Vec<String>>
    ) {
        // Initialize
        self.local = local;
        self.remote = remote;
        self.all = HashMap::new();
        self.colors = HashMap::new();
        self.sorted = Vec::new();
        self.indices = Vec::new();
        
        // Combine local and remote branches
        for (&alias, branches) in self.local.iter() {
            self.all.insert(alias, branches.clone());
        }
        for (&oidi, branches) in self.remote.iter() {
            self.all
                .entry(oidi)
                .and_modify(|existing| existing.extend(branches.iter().cloned()))
                .or_insert_with(|| branches.clone());
        }

        // Make all branches visible if none are
        if self.visible.is_empty() {
            for (&alias, branches) in self.all.iter() {
                self.visible.insert(alias, branches.clone());
            }
        }
        
        // Branch tuple vectors
        let mut local: Vec<(u32, String)> = self.local.iter().flat_map(|(&alias, branches)| {
                branches.iter().map(move |branch| (alias, branch.clone()))
            }).collect();
        let mut remote: Vec<(u32, String)> = self.remote.iter().flat_map(|(&alias, branches)| {
                branches.iter().map(move |branch| (alias, branch.clone()))
            }).collect();

        // Sorting tuples
        local.sort_by(|a, b| a.1.cmp(&b.1));
        remote.sort_by(|a, b| a.1.cmp(&b.1));

        // Combining into sorted
        self.sorted = local.into_iter().chain(remote).collect();

        // Set branch colors
        for (oidi, &lane_idx) in lanes.iter() {
            self.colors.insert(*oidi, color.borrow().get_lane(lane_idx));
        }
        
        // Build a lookup of branch aliases to positions in sorted aliases
        let mut sorted_time = self.sorted.clone();
        let index_map: std::collections::HashMap<u32, usize> = oid_manager.get_sorted_aliases().iter().enumerate().map(|(i, &oidi)| (oidi, i)).collect();

        // Sort the vector using the index map
        sorted_time.sort_by_key(|(oidi, _)| index_map.get(oidi).copied().unwrap_or(usize::MAX));
        self.indices = Vec::new();
        sorted_time.iter().for_each(|(oidi, _)| {
            self.indices.push(oid_manager.get_sorted_aliases().iter().position(|o| oidi == o).unwrap_or(usize::MAX));
        });
    }
}

pub struct App {
    // General
    pub logo: Vec<Span<'static>>,
    pub path: String,
    pub repo: Rc<Repository>,
    pub hint: String,
    pub spinner: Spinner,
    pub keymap: IndexMap<KeyBinding, Command>,
    pub last_input_direction: Option<Direction>,
    pub theme: Theme,

    // User
    pub name: String,
    pub email: String,

    // Walker utilities
    pub color: Rc<RefCell<ColorPicker>>,
    pub buffer: RefCell<Buffer>,
    pub layers: LayersContext,
    pub walker_rx: Option<std::sync::mpsc::Receiver<WalkerOutput>>,
    pub walker_cancel: Option<Arc<AtomicBool>>,
    pub walker_handle: Option<std::thread::JoinHandle<()>>,

    // Walker data
    pub oid_manager: OidManager,
    pub branch_manager: BranchManager,
    pub uncommitted: UncommittedChanges,

    // Cache
    pub current_diff: Vec<FileChange>,
    pub file_name: Option<String>,
    pub viewer_lines: Vec<ListItem<'static>>,

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

    // Settings
    pub settings_selected: usize,
    pub settings_selections: Vec<usize>,

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

    // Modal solo
    pub modal_solo_selected: i32,

    // Modal commit
    pub commit_editor: EditorState,
    pub commit_editor_event_handler: EditorEventHandler,

    // Modal create branch
    pub create_branch_editor: EditorState,
    pub create_branch_editor_event_handler: EditorEventHandler,

    // Modal delete a branch
    pub modal_delete_branch_selected: i32,

    // Exit
    pub is_exit: bool,
}

impl App  {
    
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {

        self.load_keymap();
        self.reload();

        // Main loop
        while !self.is_exit {

            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                self.handle_events()?;
            }

            // Handle background processes
            self.sync();

            // Draw the user interface
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {

        // Compute the layout
        self.layout(frame);

        let is_splash = self.viewport == Viewport::Splash;

        frame.render_widget( Block::default()
            // .title(vec![
            //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
            //     Span::styled(" graph ", Style::default().fg(if self.focus == Focus::Viewport { COLOR_GREY_500 } else { COLOR_TEXT } )),
            //     Span::styled("─", Style::default().fg(COLOR_BORDER)),
            // ])
            // .title_alignment(ratatui::layout::Alignment::Right)
            // .title_style(Style::default().fg(COLOR_GREY_400))
            .borders(if is_splash { Borders::NONE } else { Borders::ALL })
            .border_style(Style::default().fg(self.theme.COLOR_BORDER))
            .border_type(ratatui::widgets::BorderType::Rounded), self.layout.app);
                
        // Main layout
        if !is_splash {
            self.draw_title(frame);
        }

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
            Viewport::Splash => {
                self.draw_splash(frame);
            }
            Viewport::Settings => {
                self.draw_settings(frame);
            }
        }

        // Panes
        match self.viewport {
            Viewport::Splash => {}
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
        if !is_splash {
            self.draw_statusbar(frame);
        }

        // Modals
        match self.focus {
            Focus::ModalCheckout => {
                self.draw_modal_checkout(frame);
            }
            Focus::ModalSolo => {
                self.draw_modal_solo(frame);
            }
            Focus::ModalCommit => {
                self.draw_modal_commit(frame);
            }
            Focus::ModalCreateBranch => {
                self.draw_modal_create_branch(frame);
            }
            Focus::ModalDeleteBranch => {
                self.draw_modal_delete_branch(frame);
            }
            _ => {}
        }
    }

    pub fn reload(&mut self) {

        // Update colors        
        self.color = Rc::new(RefCell::new(ColorPicker::from_theme(&self.theme)));
        self.layers = layers!(Rc::new(RefCell::new(ColorPicker::from_theme(&self.theme))));
        self.logo = vec![
            Span::styled("  g", Style::default().fg(self.theme.COLOR_GRASS)),
            Span::styled("u", Style::default().fg(self.theme.COLOR_GRASS)),
            Span::styled("i", Style::default().fg(self.theme.COLOR_GRASS)),
            Span::styled("t", Style::default().fg(self.theme.COLOR_GRASS)),
            Span::styled("a", Style::default().fg(self.theme.COLOR_GRASS)),
            Span::styled("╭", Style::default().fg(self.theme.COLOR_GREEN))
        ];

        // Cancel any existing walker thread immediately
        if let Some(cancel_flag) = &self.walker_cancel {
            cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }

        // Try to join previous walker handle if present (best-effort, non-blocking)
        if let Some(handle) = self.walker_handle.take() {
            // detach by spawning a thread that joins to avoid blocking reload
            std::thread::spawn(move || {
                let _ = handle.join();
            });
        }

        // Get user credentials
        let (name, email) = get_git_user_info(&self.repo).expect("Error");
        self.name = name.unwrap();
        self.email = email.unwrap();

        // Restart the spinner
        self.spinner.start();

        // Create a new cancellation flag and channel
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel.clone();
        self.walker_cancel = Some(cancel);

        let (tx, rx) = channel();
        self.walker_rx = Some(rx);

        // Copy the repo path and visible branches
        let path = self.path.clone();
        let visible = self.branch_manager.visible.clone();

        // Spawn a thread that computes something; it will check cancel flag between iterations
        let handle = thread::spawn(move || {
            // Create the walker
            let mut walk_ctx = Walker::new(path, 10000, visible).expect("Error");
            let mut is_first_batch = true;

            // Walker loop
            loop {
                if cancel_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }

                // Parse a chunk
                let again = walk_ctx.walk();

                // Send the message to the main thread
                if tx.send(WalkerOutput {
                    oid_manager: walk_ctx.oid_manager.clone(),
                    tip_lanes: walk_ctx.tip_lanes.clone(),
                    local: walk_ctx.local.clone(),
                    remote: walk_ctx.remote.clone(),
                    buffer: walk_ctx.buffer.clone(),
                    is_first_batch,
                    again,
                }).is_err() {
                    // Receiver dropped, stop
                    break;
                }

                // Break the loop if walker finished
                if !again {
                    break;
                } else {
                    is_first_batch = false;
                }
            }
        });

        self.walker_handle = Some(handle);
    }

    pub fn sync(&mut self) {
        if let Some(rx) = &self.walker_rx && let Ok(result) = rx.try_recv() {

            // Crude check to see if this is a first iteration
            if result.is_first_batch && self.viewport == Viewport::Splash {
                self.viewport = Viewport::Graph;
            }

            // Reset utilities
            self.buffer = RefCell::new(Buffer::default());
            self.layers = layers!(self.color.clone());
            
            // Get uncomitted changes info
            self.uncommitted = get_filenames_diff_at_workdir(&self.repo).expect("Error");
            
            // Lookup tables
            self.oid_manager = result.oid_manager;

            // Buffer
            self.buffer = result.buffer;

            // Mapping of tip oids of the branches to the colors            
            self.branch_manager.feed(
                &self.oid_manager,
                &self.color,
                &result.tip_lanes,
                result.local,
                result.remote
            );

            if !result.again {
                self.spinner.stop();
            }
        }
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
