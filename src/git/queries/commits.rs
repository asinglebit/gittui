#[rustfmt::skip]
use std::collections::HashMap;
#[rustfmt::skip]
use git2::{
    BranchType,
    Oid,
    Repository,
    Time
};
#[rustfmt::skip]
use crate::{
    core::{
        walker::{
            LazyWalker
        }
    }
};

// Returns a map of commit OIDs to the branch names that point to them
pub fn get_tip_oids(repo: &Repository) -> HashMap<Oid, Vec<String>> {
    let mut tips: HashMap<Oid, Vec<String>> = HashMap::new();

    // Iterate through both local and remote branches
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            if let Some(oid) = branch.get().target() {
                // Get branch name (or "unknown" if not available)
                let name = branch.name().unwrap().unwrap_or("unknown").to_string();
                // Map each OID to one or more branch names pointing to it
                tips.entry(oid).or_default().push(name);
            }
        }
    }

    tips
}

// Outcomes:
// Update branch_oid_map: branch names to their latest commit OID
// Update the oids vector
#[allow(clippy::too_many_arguments)]
pub fn get_branches_and_sorted_oids(
    walker: &LazyWalker,
    tips: &HashMap<Oid, Vec<String>>,
    oids: &mut [Oid],
    branch_oid_map: &mut HashMap<String, Oid>,
    sorted: &mut Vec<Oid>,
    amount: usize,
) {
    // Get the next batch of commits
    let chunk = walker.next_chunk(amount);
    if chunk.is_empty() {
        // No more commits left
        return;
    }

    // Seed each tip with its branch names
    if oids.len() == 1 {
        for (oid, branches) in tips {
            for name in branches {
                branch_oid_map.entry(name.clone()).or_insert(*oid);
            }
        }
    }

    // Walk all commits topologically and propagate branch membership backwards
    for oid in chunk {
        sorted.push(oid);
    }
}

// Returns the name of the currently checked-out branch, or None if detached HEAD
pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().unwrap();
    if head.is_branch() {
        head.shorthand().map(|s| s.to_string())
    } else {
        None
    }
}

// Returns a map of commit OIDs to their timestamps:
// (commit time, committer time, author time)
pub fn get_timestamps(
    repo: &Repository,
    _branches: &HashMap<Oid, Vec<String>>,
) -> HashMap<Oid, (Time, Time, Time)> {
    _branches
        .keys()
        .map(|&sha| {
            let commit = repo.find_commit(sha).unwrap();
            let author_time = commit.author().when();
            let committer_time = commit.committer().when();
            let time = commit.time();
            // Map each OID to its associated timestamps
            (sha, (time, committer_time, author_time))
        })
        .collect()
}
