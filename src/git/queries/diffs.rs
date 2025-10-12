#[rustfmt::skip]
use std::{
    collections::{
        HashSet,
    },
    path::Path
};
#[rustfmt::skip]
use git2::{
    DiffOptions,
    ObjectType,
    Delta,
    Oid,
    Diff,
    Repository,
    StatusOptions,
    DiffFormat::{
        Patch
    }
};
#[rustfmt::skip]
use crate::{
    utils::{
        symbols::{
            decode_bytes
        }
    }
};

#[derive(Debug, Default)]
pub struct UncommittedChanges {
    pub unstaged: FileChanges,
    pub staged: FileChanges,
    pub modified_count: usize,
    pub added_count: usize,
    pub deleted_count: usize,
    pub is_clean: bool,
    pub is_staged: bool,
    pub is_unstaged: bool,
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

#[derive(Debug)]
pub struct LineChange {
    pub origin: char, // '+', '-', ' ' for added, removed, context
    pub content: String, // Line content
}

#[derive(Debug)]
pub struct Hunk {
    pub header: String, // Hunk header, e.g. @@ -X,Y +X,Y @@
    pub lines: Vec<LineChange>,
}

pub fn get_filenames_diff_at_workdir(repo: &Repository) -> Result<UncommittedChanges, git2::Error> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .show(git2::StatusShow::IndexAndWorkdir)
        .renames_head_to_index(false)
        .renames_index_to_workdir(false);

    let statuses = repo.statuses(Some(&mut options))?;
    let mut changes = UncommittedChanges::default();

    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("").to_string();

        // Skip unchanged files
        if status.is_empty() {
            continue;
        }

        // Check staged changes
        if status.is_index_modified() {
            changes.staged.modified.push(path.clone());
        }
        if status.is_index_new() {
            changes.staged.added.push(path.clone());
        }
        if status.is_index_deleted() {
            changes.staged.deleted.push(path.clone());
        }

        // Check unstaged changes
        if status.is_wt_modified() {
            changes.unstaged.modified.push(path.clone());
        }
        if status.is_wt_new() {
            changes.unstaged.added.push(path.clone());
        }
        if status.is_wt_deleted() {
            changes.unstaged.deleted.push(path.clone());
        }
    }

    // Count deduplicated
    changes.modified_count = deduplicate(&changes.staged.modified, &changes.unstaged.modified);
    changes.added_count = deduplicate(&changes.staged.added, &changes.unstaged.added);
    changes.deleted_count = deduplicate(&changes.staged.deleted, &changes.unstaged.deleted);

    // Flags
    changes.is_staged = !changes.staged.modified.is_empty() || !changes.staged.added.is_empty() || !changes.staged.deleted.is_empty();
    changes.is_unstaged = !changes.unstaged.modified.is_empty() || !changes.unstaged.added.is_empty() || !changes.unstaged.deleted.is_empty();
    changes.is_clean = !changes.is_staged && !changes.is_unstaged;

    Ok(changes)
}

pub fn get_filenames_diff_at_oid(repo: &Repository, oid: Oid) -> Vec<FileChange> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();
    let mut changes = Vec::new();

    // Handle initial commit with no parent
    if commit.parent_count() == 0 {
        walk_tree(repo, &tree, "", &mut changes);
        return changes;
    }

    // Handle rest of the commits, diffing against parents
    let parent_tree = commit.parent(0).unwrap().tree().unwrap();
    let mut opts = DiffOptions::new();
    opts.include_untracked(false)
        .recurse_untracked_dirs(false)
        .include_typechange(false)
        .ignore_submodules(true)
        .show_binary(false)
        .minimal(false)
        .skip_binary_check(true);

    let diff = repo
        .diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut opts))
        .unwrap();

    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .unwrap()
            .display()
            .to_string();

        // TODO: think of something better later.
        // We want to make sure we only recurse through folders but we want it to be cheap
        // Crude check: no '.' -> folder
        let is_folder = !path.contains('.');

        if is_folder {
            if let Ok(tree_obj) = repo.find_tree(delta.new_file().id()) {
                walk_tree(repo, &tree_obj, &path, &mut changes);
                continue;
            }
        }

        changes.push(FileChange {
            filename: path,
            status: match delta.status() {
                Delta::Added => FileStatus::Added,
                Delta::Modified => FileStatus::Modified,
                Delta::Deleted => FileStatus::Deleted,
                Delta::Renamed => FileStatus::Renamed,
                _ => FileStatus::Other,
            },
        });
    }

    changes
}

pub fn get_file_diff_at_workdir(
    repo: &Repository,
    filename: &str,
) -> Result<Vec<Hunk>, git2::Error> {
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());

    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(filename);

    diff_to_hunks(repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_options))?)
}

pub fn get_file_diff_at_oid(
    repo: &Repository,
    commit_oid: Oid,
    filename: &str,
) -> std::result::Result<Vec<Hunk>, git2::Error> {
    let commit = repo.find_commit(commit_oid)?;
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(filename);

    diff_to_hunks(repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_options))?)
}

pub fn get_file_at_oid(repo: &Repository, commit_oid: Oid, filename: &str) -> Vec<String> {
    let commit = repo.find_commit(commit_oid).unwrap();
    let tree = commit.tree().unwrap();
    tree.get_path(Path::new(filename))
        .ok()
        .and_then(|entry| repo.find_blob(entry.id()).ok())
        .map(|blob| decode_bytes(blob.content()).lines().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

pub fn get_file_at_workdir(repo: &Repository, filename: &str) -> Vec<String> {
    let full_path = repo
        .workdir()
        .map(|root| root.join(filename))
        .unwrap_or_else(|| Path::new(filename).to_path_buf());
    std::fs::read_to_string(full_path)
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}


fn deduplicate(a: &[String], b: &[String]) -> usize {
    a.iter().chain(b).collect::<HashSet<_>>().len()
}

fn walk_tree(repo: &Repository, tree: &git2::Tree, base: &str, changes: &mut Vec<FileChange>) {
    for entry in tree.iter() {
        if let Some(name) = entry.name() {
            let path = if base.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", base, name)
            };

            match entry.kind() {
                Some(ObjectType::Blob) => {
                    changes.push(FileChange {
                        filename: path,
                        status: FileStatus::Added,
                    });
                }
                Some(ObjectType::Tree) => {
                    if let Ok(subtree) = entry.to_object(repo).and_then(|o| o.peel_to_tree()) {
                        walk_tree(repo, &subtree, &path, changes);
                    }
                }
                _ => {}
            }
        }
    }
}

fn diff_to_hunks(diff: Diff) -> Result<Vec<Hunk>, git2::Error> {
    let mut hunks = Vec::new();
    diff.print(Patch, |_, hunk_opt, line| {
        if let Some(hunk) = hunk_opt {
            hunks.push(Hunk {
                header: decode_bytes(hunk.header()).to_string(),
                lines: Vec::new(),
            });
        }

        if let Some(last) = hunks.last_mut() {
            last.lines.push(LineChange {
                origin: line.origin() as char,
                content: decode_bytes(line.content()).to_string(),
            });
        }

        true
    })?;
    Ok(hunks)
}

