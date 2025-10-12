#[rustfmt::skip]
use std::{
    collections::{
        HashMap
    }
};
#[rustfmt::skip]
use git2::{
    BranchType,
    Oid,
    Repository,
    Time
};

// Returns a vector of all commit OIDs in the repository
pub fn get_sorted_oids(repo: &Repository) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();

    // Push all branch tips (local and remote) into the revwalk
    for branch in repo.branches(None).unwrap() {
        let (branch, _) = branch.unwrap();
        if let Some(oid) = branch.get().target() {
            revwalk.push(oid).unwrap();
        }
    }

    // Configure sorting: topological and time-based
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
        .unwrap();

    // Collect all OIDs from the revwalk
    revwalk.filter_map(Result::ok).collect()
}

// Returns a map of commit OIDs to the branch names that point to them
pub fn get_tip_oids(repo: &Repository) -> HashMap<Oid, Vec<String>> {
    let mut tips: HashMap<Oid, Vec<String>> = HashMap::new();

    // Iterate through both local and remote branches
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            if let Some(target) = branch.get().target() {
                // Get branch name (or "unknown" if not available)
                let name = branch.name().unwrap().unwrap_or("unknown").to_string();
                // Map each OID to one or more branch names pointing to it
                tips.entry(target).or_default().push(name);
            }
        }
    }

    tips
}

// Builds two maps:
// 1. oid_branch_map: commit OIDs to the branch names that contain them
// 2. branch_oid_map: branch names to their latest commit OID
pub fn get_branch_oids(
    repo: &Repository,
    tips: &HashMap<Oid, Vec<String>>,
) -> (HashMap<Oid, Vec<String>>, HashMap<String, Oid>) {
    let mut oid_branch_map: HashMap<Oid, Vec<String>> = HashMap::new();
    let mut branch_oid_map: HashMap<String, Oid> = HashMap::new();

    // For each branch tip, traverse its history
    for (oid_tip, names) in tips {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(*oid_tip).unwrap();

        // Walk through each commit reachable from the tip
        for oid_step in revwalk {
            let oid = oid_step.unwrap();

            // Associate each commit with the branch names that reach it
            for name in names {
                oid_branch_map.entry(oid).or_default().push(name.clone());
                // Associate each branch name with its tip commit (first seen)
                branch_oid_map.entry(name.to_string()).or_insert(oid);
            }
        }
    }

    (oid_branch_map, branch_oid_map)
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
