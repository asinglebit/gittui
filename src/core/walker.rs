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
    }
};
#[rustfmt::skip]
use git2::{
    BranchType,
    Oid,
    Repository,
    Revwalk,
};
use crate::{app::app::OidManager, core::chunk::NONE};
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
    pub fn new(
        repo: Rc<Repository>,
        visible: HashMap<u32, Vec<String>>,
        oid_manager: &mut OidManager
    ) -> Result<Self, git2::Error> {
        let revwalk = Self::build_revwalk(&repo, visible, oid_manager)?;
        Ok(Self {
            revwalk: Mutex::new(revwalk),
        })
    }

    // Reset the revwalk
    pub fn reset(
        &self,
        repo: Rc<Repository>,
        visible: HashMap<u32, Vec<String>>,
        oid_manager: &mut OidManager
    ) -> Result<(), git2::Error> {
        let revwalk = Self::build_revwalk(&repo, visible, oid_manager)?;
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
    fn build_revwalk(
        repo: &Repository,
        visible: HashMap<u32, Vec<String>>,
        oid_manager: &mut OidManager
    ) -> Result<Revwalk<'static>, git2::Error> {
        // Safe: we keep repo alive in Rc, so transmute to 'static is safe
        let repo_ref: &'static Repository =
            unsafe { std::mem::transmute::<&Repository, &'static Repository>(repo) };
        let mut revwalk = repo_ref.revwalk()?;

        // TODO: Steal faster implementation from get_tip_oids function!
        // Push all local and remote branch tips
        for branch_type in [BranchType::Local, BranchType::Remote] {
            for branch_result in repo.branches(Some(branch_type))? {
                let (branch, _) = branch_result?;
                if let Some(oid) = branch.get().target() {

                    // Get the oidi
                    let alias = oid_manager.get_alias_by_oid(oid);

                    if visible.is_empty() || visible.contains_key(&alias) {
                        revwalk.push(oid)?;
                    }
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
    
    // Lazy walker
    pub walker: LazyWalker,

    // Walker utilities
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oid_manager: OidManager,

    pub tip_lanes: HashMap<u32, usize>,
    pub local: HashMap<u32, Vec<String>>,
    pub remote: HashMap<u32, Vec<String>>,

    // Pagination
    pub amount: usize
}

// Output structure for walk results
pub struct WalkerOutput {

    // Walker utilities
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oid_manager: OidManager,

    pub tip_lanes: HashMap<u32, usize>,
    pub local: HashMap<u32, Vec<String>>,
    pub remote: HashMap<u32, Vec<String>>,

    // Pagination
    pub again: bool,
    pub is_first_batch: bool
}

impl Walker {
    // Creates a new walker
    pub fn new(
        path: String,
        amount: usize,
        visible: HashMap<u32, Vec<String>>,
    ) -> Result<Self, git2::Error> {
        let path = path.clone();
        let repo = Rc::new(Repository::open(path).expect("Failed to open repo"));
        
        // Walker utilities
        let buffer = RefCell::new(Buffer::default());

        // Walker data
        let mut oid_manager = OidManager::default();
        let tip_lanes = HashMap::new();
        let (local, remote) = get_tip_oids(&repo, &mut oid_manager);
        
        // Lazy walker
        let walker = LazyWalker::new(repo.clone(), visible, &mut oid_manager).expect("Error");

        Ok(Self {
            repo,
            
            // Lazy walker
            walker,
            
            // Walker utilities
            buffer,

            // Walker data
            oid_manager,
            tip_lanes,
            local,
            remote,

            // Pagination
            amount
        })
    }

    // Walk through "amount" commits, update buffers and render lines
    pub fn walk(&mut self) -> bool {

        // Determine current HEAD oid
        let head_oid = self.repo.head().unwrap().target().unwrap();

        // Get the oidi
        let head_alias = self.oid_manager.get_alias_by_oid(head_oid);

        // Sort commits
        let mut sorted_batch: Vec<u32> = Vec::new();
        get_branches_and_sorted_oids(
            &self.walker,
            &mut self.oid_manager,
            &mut sorted_batch,
            self.amount,
        );

        // Make a fake commit for unstaged changes
        if self.oid_manager.get_commit_count() == 1 {
            self.buffer
                .borrow_mut()
                .update(Chunk::uncommitted(head_alias, NONE));
        }

        // Go through the commits, inferring the graph
        for &alias in sorted_batch.iter() {
            let mut merger_oidi: u32 = NONE;
            let oid = self.oid_manager.get_oid_by_alias(alias);
            let commit = self.repo.find_commit(*oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();

            // Gat parent aliases
            let parent_a = if let Some(parent) = parents.first() {
                self.oid_manager.get_alias_by_oid(*parent)
            } else { NONE };
            let parent_b = if let Some(parent) = parents.get(1) {
                self.oid_manager.get_alias_by_oid(*parent)
            } else { NONE };

            let chunk = Chunk::commit(alias, parent_a, parent_b);

            let mut is_commit_found = false;
            let mut lane_idx = 0;

            // Update
            self.buffer.borrow_mut().update(chunk);

            for chunk in &self.buffer.borrow().curr {
                if !chunk.is_dummy() && alias == chunk.oidi {
                    is_commit_found = true;

                    if self.local.contains_key(&alias) || self.remote.contains_key(&alias) {
                        self.tip_lanes.insert(alias, lane_idx);
                    }

                    if chunk.parent_a != NONE && chunk.parent_b != NONE {
                        let mut is_merger_found = false;
                        for chunk_nested in &self.buffer.borrow().curr {
                            if chunk_nested.parent_a != NONE && chunk_nested.parent_b == NONE
                                && chunk.parent_b == chunk_nested.parent_a
                            {
                                is_merger_found = true;
                                break;
                            }
                        }
                        if !is_merger_found {
                            merger_oidi = chunk.oidi;
                        }
                    }
                }

                lane_idx += 1;
            }

            if !is_commit_found {
                self.tip_lanes.insert(alias, lane_idx);
            }

            // Now we can borrow mutably
            if merger_oidi != NONE {
                self.buffer.borrow_mut().merger(merger_oidi);
            }

            // Serialize
            self.oid_manager.append_sorted_alias(alias);
        }

        // Indicate whether repeats are needed
        // Too lazy to make an off by one mistake here, zero is fine
        if sorted_batch.is_empty() {
            self.buffer.borrow_mut().backup();
            return false;
        }

        true
    }
}
