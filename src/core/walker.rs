#[rustfmt::skip]
use std::{
    rc::Rc,
    sync::{
        Mutex
    },
    cell::{
        RefCell
    },
    collections::{
        HashMap,
        HashSet
    }
};
#[rustfmt::skip]
use git2::{
    BranchType,
    Oid,
    Repository,
    Revwalk,
};
#[rustfmt::skip]
use ratatui::{
    style::Color,
    text::{
        Line,
    },
};
#[rustfmt::skip]
use crate::{
    layers,
    core::{
        layers::{
            LayersContext,
        },
        renderers::{
            render_uncommitted,
            render_buffer,
            render_graph,
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
            commits::{
                get_branches_and_sorted_oids,
                get_tip_oids
            },
            diffs::{
                get_filenames_diff_at_workdir
            },
            helpers::{
                UncommittedChanges
            }
        }
    },
};

// Encapsulate a revwalk over the git repository, allowing incremental fetching of commits
pub struct LazyWalker {
    revwalk: Mutex<Revwalk<'static>>,
}

impl LazyWalker {
    // Creates a new LazyWalker by building a revwalk from the repo
    pub fn new(repo: Rc<Repository>) -> Result<Self, git2::Error> {
        let revwalk = Self::build_revwalk(&repo)?;
        Ok(Self {
            revwalk: Mutex::new(revwalk),
        })
    }

    // Reset the revwalk
    pub fn reset(&self, repo: Rc<Repository>) -> Result<(), git2::Error> {
        let revwalk = Self::build_revwalk(&repo)?;
        let mut guard = self.revwalk.lock().unwrap();
        *guard = revwalk;
        Ok(())
    }

    // Get up to "count" commits from the global revwalk
    pub fn next_chunk(&self, count: usize) -> Vec<Oid> {
        let mut revwalk = self.revwalk.lock().unwrap();
        revwalk
            .by_ref()
            .take(count)
            .filter_map(Result::ok)
            .collect()
    }

    // Internal helper to build a revwalk for all branch tips
    fn build_revwalk(repo: &Repository) -> Result<Revwalk<'static>, git2::Error> {
        // Safge: we keep repo alive in Rc, so transmute to 'static is safe
        let repo_ref: &'static Repository =
            unsafe { std::mem::transmute::<&Repository, &'static Repository>(repo) };
        let mut revwalk = repo_ref.revwalk()?;

        // Push all local and remote branch tips
        for branch_type in [BranchType::Local, BranchType::Remote] {
            for branch in repo.branches(Some(branch_type))? {
                let (branch, _) = branch?;
                if let Some(oid) = branch.get().target() {
                    revwalk.push(oid)?;
                }
            }
        }

        // Topological and chronological sorting
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        Ok(revwalk)
    }
}

// Context for walking and rendering commits
pub struct Walker {
    // General
    pub repo: Rc<Repository>,
    pub walker: LazyWalker,

    // Walker utilities
    pub color: Rc<RefCell<ColorPicker>>,
    pub buffer: RefCell<Buffer>,
    pub layers: LayersContext,

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

    // Pagination
    pub amount: usize,
}

// Output structure for walk results
pub struct WalkerOutput {
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub oid_colors: HashMap<Oid, Color>,
    pub tip_colors: HashMap<Oid, Color>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub oid_branch_map: HashMap<Oid, HashSet<String>>,
    pub uncommitted: UncommittedChanges,
    pub lines_graph: Vec<Line<'static>>,
    pub lines_branches: Vec<Line<'static>>,
    pub lines_messages: Vec<Line<'static>>,
    pub lines_buffers: Vec<Line<'static>>,
    pub again: bool, // Indicates whether more commits remain to walk
}

impl Walker {
    // Creates a new walker
    pub fn new(path: String, amount: usize) -> Result<Self, git2::Error> {
        let path = path.clone();
        let repo = Rc::new(Repository::open(path).expect("Failed to open repo"));
        let walker = LazyWalker::new(repo.clone()).expect("Error");
        let tips = get_tip_oids(&repo);
        let uncommitted = get_filenames_diff_at_workdir(&repo).expect("Error");

        Ok(Self {
            repo,
            walker,
            color: Rc::new(RefCell::new(ColorPicker::default())),
            buffer: RefCell::new(Buffer::default()),
            layers: layers!(Rc::new(RefCell::new(ColorPicker::default()))),
            oids: vec![Oid::zero()],
            tips,
            oid_colors: HashMap::new(),
            tip_colors: HashMap::new(),
            oid_branch_map: HashMap::new(),
            branch_oid_map: HashMap::new(),
            uncommitted,
            lines_graph: Vec::new(),
            lines_branches: Vec::new(),
            lines_messages: Vec::new(),
            lines_buffers: Vec::new(),
            amount,
        })
    }

    // Walk through "amount" commits, update buffers and render lines
    pub fn walk(&mut self) -> bool {
        // Determine current HEAD oid
        let head_oid = self.repo.head().unwrap().target().unwrap();
        self.oid_branch_map = HashMap::new();

        // Sort commits
        let mut sorted: Vec<Oid> = Vec::new();
        get_branches_and_sorted_oids(
            &self.repo,
            &self.walker,
            &self.tips,
            &mut self.oids,
            &mut self.oid_branch_map,
            &mut self.branch_oid_map,
            &mut sorted,
            self.amount,
        );

        // Make a fake commit for unstaged changes
        if self.oids.len() == 1 {
            render_uncommitted(
                head_oid,
                &self.uncommitted,
                &mut self.lines_graph,
                &mut self.lines_branches,
                &mut self.lines_messages,
                &mut self.lines_buffers,
            );
            self.buffer
                .borrow_mut()
                .update(Chunk::uncommitted(Some(head_oid), None));
        }

        // Go through the commits, inferring the graph
        for &oid in sorted.iter() {
            let mut merger_oid: Option<Oid> = None;

            self.layers.clear();
            let commit = self.repo.find_commit(oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();
            let chunk = Chunk::commit(Some(oid), parents.get(0).cloned(), parents.get(1).cloned());

            let mut spans_graph = Vec::new();

            // Update
            self.buffer.borrow_mut().update(chunk);

            // Iterate over the buffer chunks, rendering the graph line
            let mut is_commit_found = false;
            let mut is_merged_before = false;
            let mut lane_idx = 0;

            // for chunk in &self.buffer.borrow().curr {
            //     if chunk.is_dummy() {
            //         if let Some(prev_snapshot) = self.buffer.borrow().history.back() {
            //             if let Some(prev) = prev_snapshot.get(lane_idx) {
            //                 if (prev.parent_a.is_some() && prev.parent_b.is_none()) ||
            //                     (prev.parent_a.is_none() && prev.parent_b.is_some()) {
            //                     self.layers.commit(SYM_EMPTY, lane_idx);
            //                     self.layers.commit(SYM_EMPTY, lane_idx);
            //                     self.layers.pipe(SYM_BRANCH_UP, lane_idx);
            //                     self.layers.pipe(SYM_EMPTY, lane_idx);
            //                 } else {
            //                     self.layers.commit(SYM_EMPTY, lane_idx);
            //                     self.layers.commit(SYM_EMPTY, lane_idx);
            //                     self.layers.pipe(SYM_EMPTY, lane_idx);
            //                     self.layers.pipe(SYM_EMPTY, lane_idx);
            //                 }
            //             }
            //         }
            //     } else if Some(&oid) == chunk.oid.as_ref() {
            //         is_commit_found = true;
            //         self.oid_colors
            //             .entry(oid)
            //             .or_insert(self.color.borrow().get(lane_idx));

            //         let is_two_parents = chunk.parent_a.is_some() && chunk.parent_b.is_some();

            //         if is_two_parents && !self.tips.contains_key(&oid) {
            //             self.layers.commit(SYM_MERGE, lane_idx);
            //         } else if self.tips.contains_key(&oid) {
            //             self.color.borrow_mut().alternate(lane_idx);
            //             self.tip_colors
            //                 .insert(oid, self.color.borrow().get(lane_idx));
            //             self.layers.commit(SYM_COMMIT_BRANCH, lane_idx);
            //         } else {
            //             self.layers.commit(SYM_COMMIT, lane_idx);
            //         }
            //         self.layers.commit(SYM_EMPTY, lane_idx);
            //         self.layers.pipe(SYM_EMPTY, lane_idx);
            //         self.layers.pipe(SYM_EMPTY, lane_idx);

            //         // Check if commit is being merged into
            //         let mut is_mergee_found = false;
            //         let mut is_drawing = false;
            //         if is_two_parents {
            //             let mut is_merger_found = false;
            //             let mut merger_idx: usize = 0;

            //             for chunk_nested in &self.buffer.borrow().curr {
            //                 if ((chunk_nested.parent_a.is_some() && chunk_nested.parent_b.is_none()) ||
            //                     (chunk_nested.parent_a.is_none() && chunk_nested.parent_b.is_some()))
            //                     && chunk.parent_b.as_ref() == chunk_nested.parent_a.as_ref() {
            //                     is_merger_found = true;
            //                     break;
            //                 }
            //                 merger_idx += 1;
            //             }

            //             let mut mergee_idx: usize = 0;
            //             for chunk_nested in &self.buffer.borrow().curr {
            //                 if Some(&oid) == chunk_nested.oid.as_ref() {
            //                     break;
            //                 }
            //                 mergee_idx += 1;
            //             }

            //             for (chunk_nested_idx, chunk_nested) in
            //                 self.buffer.borrow().curr.iter().enumerate()
            //             {
            //                 if !is_mergee_found {
            //                     if Some(&oid) == chunk_nested.oid.as_ref() {
            //                         is_mergee_found = true;
            //                         if is_merger_found {
            //                             is_drawing = !is_drawing;
            //                         }
            //                         if !is_drawing {
            //                             is_merged_before = true;
            //                         }
            //                         self.layers.merge(SYM_EMPTY, merger_idx);
            //                         self.layers.merge(SYM_EMPTY, merger_idx);
            //                     } else {
            //                         // Before the commit
            //                         if !is_merger_found {
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                         } else if ((chunk_nested.parent_a.is_some() && chunk_nested.parent_b.is_none()) ||
            //                             (chunk_nested.parent_a.is_none() && chunk_nested.parent_b.is_some()))
            //                             && (
            //                                 chunk.parent_a.as_ref() == chunk_nested.parent_a.as_ref() ||
            //                                 chunk.parent_b.as_ref() == chunk_nested.parent_a.as_ref()
            //                             ) {
            //                             self.layers.merge(SYM_MERGE_RIGHT_FROM, merger_idx);
            //                             if chunk_nested_idx + 1 == mergee_idx {
            //                                 self.layers.merge(SYM_EMPTY, merger_idx);
            //                             } else {
            //                                 self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                             }
            //                             is_drawing = true;
            //                         } else if is_drawing {
            //                             if chunk_nested_idx + 1 == mergee_idx {
            //                                 self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                                 self.layers.merge(SYM_EMPTY, merger_idx);
            //                             } else {
            //                                 self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                                 self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                             }
            //                         } else {
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                         }
            //                     }
            //                 } else {
            //                     // After the commit
            //                     if is_merger_found && !is_merged_before {
            //                         if ((chunk_nested.parent_a.is_some() && chunk_nested.parent_b.is_none()) ||
            //                             (chunk_nested.parent_a.is_none() && chunk_nested.parent_b.is_some()))
            //                             && (
            //                                 chunk.parent_a.as_ref() == chunk_nested.parent_a.as_ref() ||
            //                                 chunk.parent_b.as_ref() == chunk_nested.parent_a.as_ref()
            //                             ) {
            //                             self.layers.merge(SYM_MERGE_LEFT_FROM, merger_idx);
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                             is_drawing = false;
            //                         } else if is_drawing {
            //                             self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                             self.layers.merge(SYM_HORIZONTAL, merger_idx);
            //                         } else {
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                             self.layers.merge(SYM_EMPTY, merger_idx);
            //                         }
            //                     }
            //                 }
            //             }

            //             if !is_merger_found {
            //                 // Count how many dummies in the end to get the real last element, append there
            //                 let mut idx = self.buffer.borrow().curr.len() - 1;
            //                 let mut trailing_dummies = 0;
            //                 for (i, c) in self.buffer.borrow().curr.iter().enumerate().rev() {
            //                     if !c.is_dummy() {
            //                         idx = i;
            //                         break;
            //                     } else {
            //                         trailing_dummies += 1;
            //                     }
            //                 }

            //                 let buffer = self.buffer.borrow();
            //                 let prev_snapshot = buffer.history.back();

            //                 if trailing_dummies > 0
            //                     && prev_snapshot.is_some()
            //                     && prev_snapshot.unwrap().len() > idx
            //                     && prev_snapshot.unwrap()[idx + 1].is_dummy()
            //                 {
            //                     self.color.borrow_mut().alternate(idx + 1);
            //                     self.layers.merge(SYM_BRANCH_DOWN, idx + 1);
            //                     self.layers.merge(SYM_EMPTY, idx + 1);
            //                 } else if trailing_dummies > 0 {
            //                     // Calculate how many lanes before we reach the branch character
            //                     for _ in lane_idx..idx {
            //                         self.layers.merge(SYM_HORIZONTAL, idx + 1);
            //                         self.layers.merge(SYM_HORIZONTAL, idx + 1);
            //                     }

            //                     self.layers.merge(SYM_MERGE_LEFT_FROM, idx + 1);
            //                     self.layers.merge(SYM_EMPTY, idx + 1);
            //                 } else {
            //                     self.color.borrow_mut().alternate(idx + 1);

            //                     // Calculate how many lanes before we reach the branch character
            //                     for _ in lane_idx..idx {
            //                         self.layers.merge(SYM_HORIZONTAL, idx + 1);
            //                         self.layers.merge(SYM_HORIZONTAL, idx + 1);
            //                     }

            //                     self.layers.merge(SYM_BRANCH_DOWN, idx + 1);
            //                     self.layers.merge(SYM_EMPTY, idx + 1);
            //                 }
            //                 merger_oid = chunk.oid;
            //             }
            //         }
            //     } else {
            //         self.layers.commit(SYM_EMPTY, lane_idx);
            //         self.layers.commit(SYM_EMPTY, lane_idx);
            //         if (
            //             chunk.parent_a.as_ref() == Some(&head_oid) ||
            //             chunk.parent_b.as_ref() == Some(&head_oid)
            //          ) && lane_idx == 0 {
            //             self.layers
            //                 .pipe_custom(SYM_VERTICAL_DOTTED, lane_idx, COLOR_GREY_500);
            //         } else {
            //             self.layers.pipe(SYM_VERTICAL, lane_idx);
            //         }
            //         self.layers.pipe(SYM_EMPTY, lane_idx);
            //     }

            //     lane_idx += 1;
            // }
            if !is_commit_found {
                if self.tips.contains_key(&oid) {
                    self.color.borrow_mut().alternate(lane_idx);
                    self.tip_colors
                        .insert(oid, self.color.borrow().get(lane_idx));
                    self.layers.commit(SYM_COMMIT_BRANCH, lane_idx);
                } else {
                    self.layers.commit(SYM_COMMIT, lane_idx);
                };
                self.layers.commit(SYM_EMPTY, lane_idx);
                self.layers.pipe(SYM_EMPTY, lane_idx);
                self.layers.pipe(SYM_EMPTY, lane_idx);
            }

            // Blend layers into the graph
            self.layers.bake(&mut spans_graph);

            // Now we can borrow mutably
            if let Some(oid) = merger_oid {
                self.buffer.borrow_mut().merger(oid);
            }

            // Serialize
            self.oids.push(oid);

            // Render
            render_graph(&oid, &mut self.lines_graph, spans_graph);
            // render_buffer(&self.buffer, &mut self.lines_buffers);
        }

        // Indicate whether repeats are needed
        // Too lazy to make an off by one mistake here, zero is fine
        !sorted.is_empty()
    }
}
