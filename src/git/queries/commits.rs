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

pub fn get_sorted_oids(repo: &Repository) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();

    // Push all branch tips
    for branch in repo.branches(None).unwrap() {
        let (branch, _) = branch.unwrap();
        if let Some(oid) = branch.get().target() {
            revwalk.push(oid).unwrap();
        }
    }

    // Topological + chronological order
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
        .unwrap();

    revwalk.filter_map(Result::ok).collect()
}

pub fn get_tip_oids(repo: &Repository) -> HashMap<Oid, Vec<String>> {
    let mut tips: HashMap<Oid, Vec<String>> = HashMap::new();
    for branch_type in [BranchType::Local, BranchType::Remote] {
        for branch in repo.branches(Some(branch_type)).unwrap() {
            let (branch, _) = branch.unwrap();
            if let Some(target) = branch.get().target() {
                let name = branch.name().unwrap().unwrap_or("unknown").to_string();
                tips.entry(target).or_default().push(name);
            }
        }
    }
    tips
}

pub fn get_branch_oids(
    repo: &Repository,
    tips: &HashMap<Oid, Vec<String>>,
) -> (HashMap<Oid, Vec<String>>, HashMap<String, Oid>) {
    let mut oid_branch_map: HashMap<Oid, Vec<String>> = HashMap::new();
    let mut branch_oid_map: HashMap<String, Oid> = HashMap::new();
    for (oid_tip, names) in tips {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(*oid_tip).unwrap();
        for oid_step in revwalk {
            let oid = oid_step.unwrap();
            for name in names {
                oid_branch_map.entry(oid).or_default().push(name.clone());
                branch_oid_map.entry(name.to_string()).or_insert(oid);
            }
        }
    }
    (oid_branch_map, branch_oid_map)
}

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().unwrap();
    if head.is_branch() {
        head.shorthand().map(|s| s.to_string())
    } else {
        None
    }
}

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
            (sha, (time, committer_time, author_time))
        })
        .collect()
}
