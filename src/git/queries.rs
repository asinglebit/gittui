#[rustfmt::skip]
use std::collections::HashMap;
#[rustfmt::skip]
use git2::{
    BranchType,
    Delta,
    Oid,
    Repository,
    Status,
    StatusOptions,
    Time
};
#[rustfmt::skip]
use ratatui::{
    style::Style,
    text::{
        Line,
        Span,
        Text
    },
};
#[rustfmt::skip]
use crate::utils::colors::{
    COLOR_GREEN,
    COLOR_GREY_400,
    COLOR_RED,
    COLOR_TEXT
};

pub fn get_uncommitted_changes_count(repo: &Repository) -> (usize, usize, usize) {
    let mut options = StatusOptions::new();
    options.include_untracked(true); // include untracked files
    options.include_ignored(false); // skip ignored files
    options.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut options)).unwrap();

    let mut new_count = 0;
    let mut modified_count = 0;
    let mut deleted_count = 0;

    for entry in statuses.iter() {
        let status = entry.status();
        if status.contains(Status::WT_NEW) || status.contains(Status::INDEX_NEW) {
            new_count += 1;
        }
        if status.contains(Status::WT_MODIFIED) || status.contains(Status::INDEX_MODIFIED) {
            modified_count += 1;
        }
        if status.contains(Status::WT_DELETED) || status.contains(Status::INDEX_DELETED) {
            deleted_count += 1;
        }
    }

    (new_count, modified_count, deleted_count)
}

pub fn get_changed_filenames_as_text(repo: &Repository, oid: Oid) -> Text<'_> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();

    let mut lines = Vec::new();

    if commit.parent_count() == 0 {
        // Initial commit — list all files
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                lines.push(Line::from(Span::styled(
                    name.to_string(),
                    Style::default().fg(COLOR_GREY_400),
                )));
            }
            git2::TreeWalkResult::Ok
        })
        .unwrap();
    } else {
        // Normal commits — diff against first parent
        let parent = commit.parent(0).unwrap();
        let parent_tree = parent.tree().unwrap();
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)
            .unwrap();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    lines.push(Line::from(Span::styled(
                        path.display().to_string(),
                        Style::default().fg(match delta.status() {
                            Delta::Added => COLOR_GREEN,
                            Delta::Deleted => COLOR_RED,
                            Delta::Modified => COLOR_TEXT,
                            Delta::Renamed => COLOR_TEXT,
                            Delta::Copied => COLOR_TEXT,
                            Delta::Untracked => COLOR_TEXT,
                            _ => COLOR_TEXT,
                        }),
                    )));
                }
                true
            },
            None,
            None,
            None,
        )
        .unwrap();
    }

    Text::from(lines)
}

pub fn get_sorted_commits(repo: &Repository) -> Vec<Oid> {
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

pub fn get_tips(repo: &Repository) -> HashMap<Oid, Vec<String>> {
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

pub fn get_branches(
    repo: &Repository,
    tips: &HashMap<Oid, Vec<String>>,
) -> HashMap<Oid, Vec<String>> {
    let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
    for (sha_tip, names) in tips {
        let mut revwalk = repo.revwalk().unwrap();
        revwalk.push(*sha_tip).unwrap();
        for sha_step in revwalk {
            let sha = sha_step.unwrap();
            for name in names {
                map.entry(sha).or_default().push(name.clone());
            }
        }
    }
    map
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
