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

#[derive(Debug, Default)]
pub struct UncommittedChanges {
    pub unstaged: FileChanges,
    pub staged: FileChanges,
}

#[derive(Debug, Default)]
pub struct FileChanges {
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
}

#[derive(Debug)]
pub struct FileChange {
    pub filename: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Copy)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Other,
}

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

pub fn get_uncommitted_changes(repo: &Repository) -> Result<UncommittedChanges, git2::Error> {
    let mut options = StatusOptions::new();
    options.include_untracked(true);
    options.show(git2::StatusShow::IndexAndWorkdir);

    let statuses = repo.statuses(Some(&mut options))?;

    let mut changes = UncommittedChanges::default();

    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("").to_string();

        // Skip files that are committed (no changes)
        if status.is_empty() {
            continue;
        }

        // Check staged changes (INDEX vs HEAD)
        if status.is_index_modified() {
            changes.staged.modified.push(path.clone());
        } else if status.is_index_new() {
            changes.staged.added.push(path.clone());
        } else if status.is_index_deleted() {
            changes.staged.deleted.push(path.clone());
        }

        // Check unstaged changes (WORKDIR vs INDEX)
        if status.is_wt_modified() {
            changes.unstaged.modified.push(path.clone());
        } else if status.is_wt_new() {
            changes.unstaged.added.push(path);
        } else if status.is_wt_deleted() {
            changes.unstaged.deleted.push(path);
        }
    }

    Ok(changes)
}

pub fn get_changed_filenames(repo: &Repository, oid: Oid) -> Vec<FileChange> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();
    let mut changes = Vec::new();

    if commit.parent_count() == 0 {
        // Initial commit — list all files as "added"
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if let Some(name) = entry.name() {
                changes.push(FileChange {
                    filename: name.to_string(),
                    status: FileStatus::Added,
                });
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
                if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                    let status = match delta.status() {
                        Delta::Added => FileStatus::Added,
                        Delta::Deleted => FileStatus::Deleted,
                        Delta::Modified => FileStatus::Modified,
                        Delta::Renamed => FileStatus::Renamed,
                        _ => FileStatus::Other,
                    };

                    changes.push(FileChange {
                        filename: path.display().to_string(),
                        status,
                    });
                }
                true
            },
            None,
            None,
            None,
        )
        .unwrap();
    }

    changes
}
