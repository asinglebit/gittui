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
use crate::core::chunk::NONE;
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
        visible_branches: HashMap<u32, Vec<String>>,
        oidi_to_oid: &mut Vec<Oid>,
        oid_to_oidi: &mut HashMap<Oid, u32>,
    ) -> Result<Self, git2::Error> {
        let revwalk = Self::build_revwalk(&repo, visible_branches, oidi_to_oid, oid_to_oidi)?;
        Ok(Self {
            revwalk: Mutex::new(revwalk),
        })
    }

    // Reset the revwalk
    pub fn reset(
        &self,
        repo: Rc<Repository>,
        visible_branches: HashMap<u32, Vec<String>>,
        oidi_to_oid: &mut Vec<Oid>,
        oid_to_oidi: &mut HashMap<Oid, u32>,
    ) -> Result<(), git2::Error> {
        let revwalk = Self::build_revwalk(&repo, visible_branches, oidi_to_oid, oid_to_oidi)?;
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
        visible_branches: HashMap<u32, Vec<String>>,
        oidi_to_oid: &mut Vec<Oid>,
        oid_to_oidi: &mut HashMap<Oid, u32>
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
                    let oidi = *oid_to_oidi.entry(oid).or_insert_with(|| {
                        oidi_to_oid.push(oid);
                        oidi_to_oid.len() as u32 - 1
                    });

                    if visible_branches.is_empty() || visible_branches.contains_key(&oidi) {
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
    pub oidi_to_oid: Vec<Oid>,
    pub oid_to_oidi: HashMap<Oid, u32>,
    pub oidi_sorted: Vec<u32>,
    pub tip_lanes: HashMap<u32, usize>,
    pub tips_local: HashMap<u32, Vec<String>>,
    pub tips_remote: HashMap<u32, Vec<String>>,
    pub branch_oid_map: HashMap<String, u32>,

    // Pagination
    pub amount: usize

}

// Output structure for walk results
pub struct WalkerOutput {

    // Walker utilities
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oidi_to_oid: Vec<Oid>,
    pub oid_to_oidi: HashMap<Oid, u32>,
    pub oidi_sorted: Vec<u32>,
    pub tip_lanes: HashMap<u32, usize>,
    pub tips_local: HashMap<u32, Vec<String>>,
    pub tips_remote: HashMap<u32, Vec<String>>,
    pub branch_oid_map: HashMap<String, u32>,

    // Pagination
    pub again: bool,
    pub is_first_batch: bool
}

impl Walker {
    // Creates a new walker
    pub fn new(
        path: String,
        amount: usize,
        visible_branches: HashMap<u32, Vec<String>>,
    ) -> Result<Self, git2::Error> {
        let path = path.clone();
        let repo = Rc::new(Repository::open(path).expect("Failed to open repo"));
        
        // Walker utilities
        let buffer = RefCell::new(Buffer::default());

        // Walker data
        let mut oidi_to_oid: Vec<Oid> = Vec::new();
        let mut oid_to_oidi: HashMap<Oid, u32> = HashMap::new();
        let oidi_sorted = vec![NONE];
        let tip_lanes = HashMap::new();
        let (tips_local, tips_remote) = get_tip_oids(&repo, &mut oidi_to_oid, &mut oid_to_oidi);
        let branch_oid_map: HashMap<String, u32> = HashMap::new();
        
        // Lazy walker
        let walker = LazyWalker::new(repo.clone(), visible_branches, &mut oidi_to_oid, &mut oid_to_oidi).expect("Error");

        Ok(Self {
            repo,
            
            // Lazy walker
            walker,
            
            // Walker utilities
            buffer,

            // Walker data
            oidi_to_oid,
            oid_to_oidi,
            oidi_sorted,
            tip_lanes,
            tips_local,
            tips_remote,
            branch_oid_map,

            // Pagination
            amount
        })
    }

    // Walk through "amount" commits, update buffers and render lines
    pub fn walk(&mut self) -> bool {

        // Determine current HEAD oid
        let head_oid = self.repo.head().unwrap().target().unwrap();

        // Get the oidi
        let head_oidi = *self.oid_to_oidi.entry(head_oid).or_insert_with(|| {
            self.oidi_to_oid.push(head_oid);
            self.oidi_to_oid.len() as u32 - 1
        });

        // Sort commits
        let mut sorted: Vec<u32> = Vec::new();
        get_branches_and_sorted_oids(
            &self.walker,
            &self.tips_local,
            &self.tips_remote,
            &mut self.oidi_sorted,
            &mut self.oidi_to_oid,
            &mut self.oid_to_oidi,
            &mut self.branch_oid_map,
            &mut sorted,
            self.amount,
        );

        // Make a fake commit for unstaged changes
        if self.oidi_sorted.len() == 1 {
            self.buffer
                .borrow_mut()
                .update(Chunk::uncommitted(head_oidi, NONE));
        }

        // Go through the commits, inferring the graph
        for &oidi in sorted.iter() {
            let mut merger_oidi: u32 = NONE;
            let oid = self.oidi_to_oid.get(oidi as usize).unwrap();
            let commit = self.repo.find_commit(*oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();

            let parent_a = if let Some(parent) = parents.first() {
                // Get the oidi
                *self.oid_to_oidi.entry(*parent).or_insert_with(|| {
                    self.oidi_to_oid.push(*parent);
                    self.oidi_to_oid.len() as u32 - 1
                })
            } else { NONE };

            let parent_b = if let Some(parent) = parents.get(1) {
                // Get the oidi
                *self.oid_to_oidi.entry(*parent).or_insert_with(|| {
                    self.oidi_to_oid.push(*parent);
                    self.oidi_to_oid.len() as u32 - 1
                })
            } else { NONE };

            let chunk = Chunk::commit(oidi, parent_a, parent_b);

            let mut is_commit_found = false;
            let mut lane_idx = 0;

            // Update
            self.buffer.borrow_mut().update(chunk);

            for chunk in &self.buffer.borrow().curr {
                if !chunk.is_dummy() && oidi == chunk.oidi {
                    is_commit_found = true;

                    if self.tips_local.contains_key(&oidi) || self.tips_remote.contains_key(&oidi) {
                        self.tip_lanes.insert(oidi, lane_idx);
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
                self.tip_lanes.insert(oidi, lane_idx);
            }

            // Now we can borrow mutably
            if merger_oidi != NONE {
                self.buffer.borrow_mut().merger(merger_oidi);
            }

            // Serialize
            self.oidi_sorted.push(oidi);
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
