#[rustfmt::skip]
use std::collections::HashMap;
#[rustfmt::skip]
use git2::{
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
pub fn get_tip_oids(repo: &Repository, oidi_to_oid: &mut Vec<Oid>, oid_to_oidi: &mut HashMap<Oid, u32>) -> (HashMap<u32, Vec<String>>, HashMap<u32, Vec<String>>) {
    
    let mut tips_local: HashMap<u32, Vec<String>> = HashMap::new();
    let mut tips_remote: HashMap<u32, Vec<String>> = HashMap::new();

    // Iterate all refs once
    for reference in repo.references().unwrap().flatten() {
        // Only handle direct refs (skip symbolic ones like HEAD)
        if let Some(oid) = reference.target() {
            
            // Get the oidi
            let oidi = *oid_to_oidi.entry(oid).or_insert_with(|| {
                oidi_to_oid.push(oid);
                oidi_to_oid.len() as u32 - 1
            });

            let name = reference.name().unwrap_or("unknown");

            if let Some(stripped) = name.strip_prefix("refs/heads/") {
                tips_local.entry(oidi).or_default().push(stripped.to_string());
            } else if let Some(stripped) = name.strip_prefix("refs/remotes/") {
                tips_remote.entry(oidi).or_default().push(stripped.to_string());
            }
        }
    }

    (tips_local, tips_remote)
}

// Outcomes:
// Update branch_oid_map: branch names to their latest commit OID
// Update the oids vector
#[allow(clippy::too_many_arguments)]
pub fn get_branches_and_sorted_oids(
    walker: &LazyWalker,
    tips_local: &HashMap<u32, Vec<String>>,
    tips_remote: &HashMap<u32, Vec<String>>,
    oidi_sorted: &mut Vec<u32>,
    oidi_to_oid: &mut Vec<Oid>,
    oid_to_oidi: &mut HashMap<Oid, u32>,
    branch_oid_map: &mut HashMap<String, u32>,
    sorted: &mut Vec<u32>,
    amount: usize,
) {
    // Get the next batch of commits
    let chunk = walker.next_chunk(amount);
    if chunk.is_empty() {
        // No more commits left
        return;
    }

    // Seed each tip with its branch names
    if oidi_sorted.len() == 1 {
        for (oidi, branches) in tips_local.iter().chain(tips_remote) {
            for name in branches {
                branch_oid_map.entry(name.clone()).or_insert(*oidi);
            }
        }
    }

    // Walk all commits topologically and propagate branch membership backwards
    for oid in chunk {

        // Get the oidi
        let oidi = *oid_to_oidi.entry(oid).or_insert_with(|| {
            oidi_to_oid.push(oid);
            oidi_to_oid.len() as u32 - 1
        });

        sorted.push(oidi);
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

pub fn get_git_user_info(
    repo: &Repository,
) -> Result<(Option<String>, Option<String>), git2::Error> {
    let config = repo.config()?;
    let name = config.get_string("user.name").ok();
    let email = config.get_string("user.email").ok();
    Ok((name, email))
}
