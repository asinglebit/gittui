#[rustfmt::skip]
use std::{
    rc::Rc,
    cell::{
        RefCell
    },
    collections::{
        HashMap,
    }
};
#[rustfmt::skip]
use git2::{
    Oid,
    Repository
};
#[rustfmt::skip]
use crate::{
    core::{
        oids::{
            Oids
        },
        buffer::{
            Buffer
        },
        chunk::{
            Chunk,
            NONE
        },
        batcher::{
            Batcher
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

// Context for walking and rendering commits
pub struct Walker {
    // General
    pub repo: Rc<Repository>,
    
    // Batcher
    pub batcher: Batcher,

    // Walker utilities
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oids: Oids,

    pub branches_lanes: HashMap<u32, usize>,
    pub branches_local: HashMap<u32, Vec<String>>,
    pub branches_remote: HashMap<u32, Vec<String>>,

    // Batching
    pub amount: usize
}

// Output structure for walk results
pub struct WalkerOutput {

    // Walker utilities
    pub buffer: RefCell<Buffer>,

    // Walker data
    pub oids: Oids,

    pub branches_lanes: HashMap<u32, usize>,
    pub branches_local: HashMap<u32, Vec<String>>,
    pub branches_remote: HashMap<u32, Vec<String>>,

    // Batching
    pub is_again: bool,
    pub is_first: bool
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
        let mut oids = Oids::default();
        let branches_lanes = HashMap::new();
        let (branches_local, branches_remote) = get_tip_oids(&repo, &mut oids);
        
        // Batcher
        let batcher = Batcher::new(repo.clone(), visible, &mut oids).expect("Error");

        Ok(Self {
            repo,
            
            // Batcher
            batcher,
            
            // Walker utilities
            buffer,

            // Walker data
            oids,
            branches_lanes,
            branches_local,
            branches_remote,

            // Pagination
            amount
        })
    }

    // Walk through "amount" commits, update buffers and render lines
    pub fn walk(&mut self) -> bool {

        // Determine current HEAD oid
        let head_oid = self.repo.head().unwrap().target().unwrap();

        // Get the alias
        let head_alias = self.oids.get_alias_by_oid(head_oid);

        // Sort commits
        let mut sorted_batch: Vec<u32> = Vec::new();
        get_branches_and_sorted_oids(
            &self.batcher,
            &mut self.oids,
            &mut sorted_batch,
            self.amount,
        );

        // Make a fake commit for unstaged changes
        if self.oids.get_commit_count() == 1 {
            self.buffer
                .borrow_mut()
                .update(Chunk::uncommitted(head_alias, NONE));
        }

        // Go through the commits, inferring the graph
        for &alias in sorted_batch.iter() {
            let mut merger_alias: u32 = NONE;
            let oid = self.oids.get_oid_by_alias(alias);
            let commit = self.repo.find_commit(*oid).unwrap();
            let parents: Vec<Oid> = commit.parent_ids().collect();

            // Gat parent aliases
            let parent_a = if let Some(parent) = parents.first() {
                self.oids.get_alias_by_oid(*parent)
            } else { NONE };
            let parent_b = if let Some(parent) = parents.get(1) {
                self.oids.get_alias_by_oid(*parent)
            } else { NONE };

            let chunk = Chunk::commit(alias, parent_a, parent_b);

            let mut is_commit_found = false;
            let mut lane_idx = 0;

            // Update
            self.buffer.borrow_mut().update(chunk);

            for chunk in &self.buffer.borrow().curr {
                if !chunk.is_dummy() && alias == chunk.alias {
                    is_commit_found = true;

                    if self.branches_local.contains_key(&alias) || self.branches_remote.contains_key(&alias) {
                        self.branches_lanes.insert(alias, lane_idx);
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
                            merger_alias = chunk.alias;
                        }
                    }
                }

                lane_idx += 1;
            }

            if !is_commit_found {
                self.branches_lanes.insert(alias, lane_idx);
            }

            // Now we can borrow mutably
            if merger_alias != NONE {
                self.buffer.borrow_mut().merger(merger_alias);
            }

            // Serialize
            self.oids.append_sorted_alias(alias);
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
