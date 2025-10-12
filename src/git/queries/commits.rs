#[rustfmt::skip]
use std::{
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
    Time,
    Sort
};
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
// Update oid_branch_map: commit OIDs to the branch names that contain them
// Update branch_oid_map: branch names to their latest commit OID
// Update the oids vector
pub fn get_branches_and_sorted_oids(
    repo: &Repository,
    walker: &LazyWalker,
    tips: &HashMap<Oid, Vec<String>>,
    oids: &mut Vec<Oid>,
    oid_branch_map: &mut HashMap<Oid, HashSet<String>>,
    branch_oid_map: &mut HashMap<String, Oid>,
    sorted: &mut Vec<Oid>,
) {

    // Prepare revwalk with all branch tips
    let mut revwalk = repo.revwalk().unwrap();
    for tip_oid in tips.keys() {
        revwalk.push(*tip_oid).unwrap();
    }
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME).unwrap();

    // Seed each tip with its branch names
    if oids.len() == 1 {
        for (oid, branches) in tips {
            for name in branches {
                branch_oid_map.entry(name.clone()).or_insert(*oid);
            }
            oid_branch_map
                .entry(*oid)
                .or_default()
                .extend(branches.iter().cloned());
        }
    }

    // Walk all commits topologically and propagate branch membership backwards
    for oid_result in revwalk {
        let oid = oid_result.unwrap();
        sorted.push(oid);

        // Get the branch names that currently reach this commit
        let branches_here = oid_branch_map
            .get(&oid)
            .cloned()
            .unwrap_or_default();

        // Propagate those branch names to parents
        let commit = repo.find_commit(oid).unwrap();
        for i in 0..commit.parent_count() {
            let parent_oid = commit.parent_id(i).unwrap();
            oid_branch_map
                .entry(parent_oid)
                .or_default()
                .extend(branches_here.iter().cloned());
        }
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
