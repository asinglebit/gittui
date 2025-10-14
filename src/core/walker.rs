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
        HashMap
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
use crate::{
    core::{
        buffer::{
            Buffer
        },
        chunk::{
            Chunk
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
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub uncommitted: UncommittedChanges,

    // Pagination
    pub amount: usize,
}

// Output structure for walk results
pub struct WalkerOutput {
    pub oids: Vec<Oid>,
    pub tips: HashMap<Oid, Vec<String>>,
    pub branch_oid_map: HashMap<String, Oid>,
    pub uncommitted: UncommittedChanges,
    pub again: bool,
    pub buffer: RefCell<Buffer>,
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
            buffer: RefCell::new(Buffer::default()),
            oids: vec![Oid::zero()],
            tips,
            branch_oid_map: HashMap::new(),
            uncommitted,
            amount,
        })
    }

    // Walk through "amount" commits, update buffers and render lines
    pub fn walk(&mut self) -> bool {
        // Determine current HEAD oid
        let head_oid = self.repo.head().unwrap().target().unwrap();

        // Sort commits
        let mut sorted: Vec<Oid> = Vec::new();
        get_branches_and_sorted_oids(
            &self.walker,
            &self.tips,
            &mut self.oids,
            &mut self.branch_oid_map,
            &mut sorted,
            self.amount,
        );

        // Make a fake commit for unstaged changes
        if self.oids.len() == 1 {
            self.buffer
                .borrow_mut()
                .update(Chunk::uncommitted(Some(head_oid), None));
        }

        // Go through the commits, inferring the graph
        for &oid in sorted.iter() {
            let mut merger_oid: Option<Oid> = None;
            let commit = self.repo.find_commit(oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();
            let chunk = Chunk::commit(Some(oid), parents.get(0).cloned(), parents.get(1).cloned());

            // Update
            self.buffer.borrow_mut().update(chunk);

            for chunk in &self.buffer.borrow().curr {
                if !chunk.is_dummy() && Some(&oid) == chunk.oid.as_ref() {
                    if chunk.parent_a.is_some() && chunk.parent_b.is_some() {
                        let mut is_merger_found = false;
                        for chunk_nested in &self.buffer.borrow().curr {
                            if ((chunk_nested.parent_a.is_some()
                                && chunk_nested.parent_b.is_none())
                                || (chunk_nested.parent_a.is_none()
                                    && chunk_nested.parent_b.is_some()))
                                && chunk.parent_b.as_ref() == chunk_nested.parent_a.as_ref()
                            {
                                is_merger_found = true;
                                break;
                            }
                        }
                        if !is_merger_found {
                            merger_oid = chunk.oid;
                        }
                    }
                }
            }

            // Now we can borrow mutably
            if let Some(oid) = merger_oid {
                self.buffer.borrow_mut().merger(oid);
            }

            // Serialize
            self.oids.push(oid);
        }

        // Indicate whether repeats are needed
        // Too lazy to make an off by one mistake here, zero is fine
        if sorted.is_empty() {
            self.buffer.borrow_mut().backup();
            return false;
        }
        true
    }
}
