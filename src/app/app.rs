#[rustfmt::skip]
use std::{
    cell::{
        Cell,
        RefCell
    },
    sync::{
        Arc
    },
    collections::{
        HashMap,
        HashSet
    },
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
#[rustfmt::skip]
use crate::{
    layers,
    core::{
        layers::{
            LayersCtx,
        },
        walker::{
            LazyWalker
        },
        renderers::{
            render_uncommitted,
            render_branches,
            render_buffer,
            render_graph,
            render_messages
        },
        buffer::{
            Buffer
        },
        chunk::{
            Chunk
        }
    },
    helpers::{
        symbols::*,
        palette::*,
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
                get_branches_and_sorted_oids,
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

        while !self.is_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
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

        // Utilities
        let color = RefCell::new(ColorPicker::default());
        let buffer = RefCell::new(Buffer::default());
        let mut layers: LayersCtx = layers!(&color);

        let head_oid = self.repo.head().unwrap().target().unwrap();
        let mut sorted: Vec<Oid> = Vec::new();
        get_branches_and_sorted_oids(&self.repo, &self.walker, &self.tips, &mut self.oids, &mut self.oid_branch_map, &mut self.branch_oid_map, &mut sorted);

        // Make a fake commit for unstaged changes
        render_uncommitted(
            head_oid,
            &self.uncommitted,
            &mut self.lines_graph,
            &mut self.lines_branches,
            &mut self.lines_messages,
            &mut self.lines_buffers,
        );
        buffer
            .borrow_mut()
            .update(Chunk::uncommitted(vec![head_oid]));

        // Go through the commits, inferring the graph
        for oid in sorted {
            let mut merger_oid = None;

            layers.clear();
            let commit = self.repo.find_commit(oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();
            let chunk = Chunk::commit(oid, parents);

            let mut spans_graph = Vec::new();

            // Update
            buffer.borrow_mut().update(chunk);

            // Iterate over the buffer chunks, rendering the graph line
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;
            for chunk in &buffer.borrow().curr {
                if chunk.is_dummy() {
                    if let Some(prev) = buffer.borrow().prev.get(lane_idx) {
                        if prev.parents.len() == 1 {
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
                } else if oid == chunk.oid {
                    is_commit_found = true;
                    self.oid_colors
                        .entry(oid)
                        .or_insert(color.borrow().get(lane_idx));

                    if chunk.parents.len() > 1 && !self.tips.contains_key(&oid) {
                        layers.commit(SYM_MERGE, lane_idx);
                    } else if self.tips.contains_key(&oid) {
                        color.borrow_mut().alternate(lane_idx);
                        self.tip_colors.insert(oid, color.borrow().get(lane_idx));
                        layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                    } else {
                        layers.commit(SYM_COMMIT, lane_idx);
                    }
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.pipe(SYM_EMPTY, lane_idx);
                    layers.pipe(SYM_EMPTY, lane_idx);

                    // Check if commit is being merged into
                    let mut is_mergee_found = false;
                    let mut is_drawing = false;
                    if chunk.parents.len() > 1 {
                        let mut is_merger_found = false;
                        let mut merger_idx: usize = 0;
                        for chunk_nested in &buffer.borrow().curr {
                            if chunk_nested.parents.len() == 1
                                && chunk.parents.last().unwrap()
                                    == chunk_nested.parents.first().unwrap()
                            {
                                is_merger_found = true;
                                break;
                            }
                            merger_idx += 1;
                        }

                        let mut mergee_idx: usize = 0;
                        for chunk_nested in &buffer.borrow().curr {
                            if oid == chunk_nested.oid {
                                break;
                            }
                            mergee_idx += 1;
                        }

                        for (chunk_nested_idx, chunk_nested) in
                            buffer.borrow().curr.iter().enumerate()
                        {
                            if !is_mergee_found {
                                if oid == chunk_nested.oid {
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
                                    } else if chunk_nested.parents.len() == 1
                                        && chunk
                                            .parents
                                            .contains(chunk_nested.parents.first().unwrap())
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
                                    if chunk_nested.parents.len() == 1
                                        && chunk
                                            .parents
                                            .contains(chunk_nested.parents.first().unwrap())
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
                            let mut idx = buffer.borrow().curr.len() - 1;
                            let mut trailing_dummies = 0;
                            for (i, c) in buffer.borrow().curr.iter().enumerate().rev() {
                                if !c.is_dummy() {
                                    idx = i;
                                    break;
                                } else {
                                    trailing_dummies += 1;
                                }
                            }

                            if trailing_dummies > 0
                                && buffer.borrow().prev.len() > idx
                                && buffer.borrow().prev[idx + 1].is_dummy()
                            {
                                color.borrow_mut().alternate(idx + 1);
                                layers.merge(SYM_BRANCH_DOWN, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            } else if trailing_dummies > 0 {
                                // color.alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                }

                                layers.merge(SYM_MERGE_LEFT_FROM, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            } else {
                                color.borrow_mut().alternate(idx + 1);

                                // Calculate how many lanes before we reach the branch character
                                for _ in lane_idx..idx {
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                    layers.merge(SYM_HORIZONTAL, idx + 1);
                                }

                                layers.merge(SYM_BRANCH_DOWN, idx + 1);
                                layers.merge(SYM_EMPTY, idx + 1);
                            }
                            merger_oid = Some(chunk.oid);
                        }
                    }
                } else {
                    layers.commit(SYM_EMPTY, lane_idx);
                    layers.commit(SYM_EMPTY, lane_idx);
                    if chunk.parents.contains(&head_oid) && lane_idx == 0 {
                        layers.pipe_custom(SYM_VERTICAL_DOTTED, lane_idx, COLOR_GREY_500);
                    } else {
                        layers.pipe(SYM_VERTICAL, lane_idx);
                    }
                    layers.pipe(SYM_EMPTY, lane_idx);
                }

                lane_idx += 1;
            }
            if !is_commit_found {
                if self.tips.contains_key(&oid) {
                    color.borrow_mut().alternate(lane_idx);
                    self.tip_colors.insert(oid, color.borrow().get(lane_idx));
                    layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                } else {
                    layers.commit(SYM_COMMIT, lane_idx);
                };
                layers.commit(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);
                layers.pipe(SYM_EMPTY, lane_idx);
            }

            // Blend layers into the graph
            layers.bake(&mut spans_graph);

            // Now we can borrow mutably
            if let Some(sha) = merger_oid {
                buffer.borrow_mut().merger(sha);
            }
            buffer.borrow_mut().backup();

            // Serialize
            self.oids.push(oid);

            // Render
            render_graph(&oid, &mut self.lines_graph, spans_graph);
            render_branches(&oid, &mut self.lines_branches, &self.tips, &self.tip_colors, &commit);
            render_messages(&commit, &mut self.lines_messages);
            render_buffer(&buffer, &mut self.lines_buffers);
        }
    }

    pub fn exit(&mut self) {
        self.is_exit = true;
    }
}
