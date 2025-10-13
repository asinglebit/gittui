#[rustfmt::skip]
use std::{
    path::Path
};
#[rustfmt::skip]
use git2::{
    Error,
    DiffOptions,
    Delta,
    Oid,
    Repository,
    StatusOptions
};
#[rustfmt::skip]
use crate::{
    helpers::{
        text::{
            decode,
            sanitize
        }
    },
    git::{
        queries::{
            helpers::{
                UncommittedChanges,
                FileChange,
                FileStatus,
                Hunk,
                deduplicate,
                diff_to_hunks,
                walk_tree
            }
        }
    }
};

// Collects and categorizes uncommitted changes in the working directory and index
pub fn get_filenames_diff_at_workdir(repo: &Repository) -> Result<UncommittedChanges, Error> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .show(git2::StatusShow::IndexAndWorkdir)
        .renames_head_to_index(false)
        .renames_index_to_workdir(false);

    // Retrieve the current status of the working directory and index
    let statuses = repo.statuses(Some(&mut options))?;
    let mut changes = UncommittedChanges::default();

    // Iterate through each file entry in the status list
    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("").to_string();

        // Skip unchanged files
        if status.is_empty() {
            continue;
        }

        // Record staged changes (index vs HEAD)
        if status.is_index_modified() {
            changes.staged.modified.push(path.clone());
        }
        if status.is_index_new() {
            changes.staged.added.push(path.clone());
        }
        if status.is_index_deleted() {
            changes.staged.deleted.push(path.clone());
        }

        // Record unstaged changes (workdir vs index)
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

    // Compute counts of deduplicated filenames
    changes.modified_count = deduplicate(&changes.staged.modified, &changes.unstaged.modified);
    changes.added_count = deduplicate(&changes.staged.added, &changes.unstaged.added);
    changes.deleted_count = deduplicate(&changes.staged.deleted, &changes.unstaged.deleted);

    // Set flags for change states
    changes.is_staged = !changes.staged.modified.is_empty()
        || !changes.staged.added.is_empty()
        || !changes.staged.deleted.is_empty();
    changes.is_unstaged = !changes.unstaged.modified.is_empty()
        || !changes.unstaged.added.is_empty()
        || !changes.unstaged.deleted.is_empty();
    changes.is_clean = !changes.is_staged && !changes.is_unstaged;

    Ok(changes)
}

// Lists all files changed in a given commit compared to its parent
pub fn get_filenames_diff_at_oid(repo: &Repository, oid: Oid) -> Vec<FileChange> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();
    let mut changes = Vec::new();

    // Handle the initial commit (no parent)
    if commit.parent_count() == 0 {
        walk_tree(repo, &tree, "", &mut changes);
        return changes;
    }

    // Diff current commit tree against its parent tree
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

    // Iterate through all deltas (changed files)
    for delta in diff.deltas() {
        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .unwrap()
            .display()
            .to_string();

        // Rough check for folders (no '.' in name)
        let is_folder = !path.contains('.');

        // Recursively collect folder contents if applicable
        if is_folder {
            if let Ok(tree_obj) = repo.find_tree(delta.new_file().id()) {
                walk_tree(repo, &tree_obj, &path, &mut changes);
                continue;
            }
        }

        // Record file and its change status
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

// Generate a line-by-line diff for a file in the working directory
pub fn get_file_diff_at_workdir(
    repo: &Repository,
    filename: &str,
) -> Result<Vec<Hunk>, git2::Error> {
    // Get the current HEAD tree (if available)
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());

    // Set diff options to include only the target file
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(filename);

    // Compare HEAD tree with workdir + index
    diff_to_hunks(
        repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_options))?,
    )
}

// Generate a line-by-line diff for a file between a commit and its parent
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

    // Diff options limited to the specific file
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(filename);

    // Compare parent tree with current commit tree
    diff_to_hunks(repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        Some(&mut diff_options),
    )?)
}

// Retrieve the contents of a file at a specific commit
pub fn get_file_at_oid(repo: &Repository, commit_oid: Oid, filename: &str) -> Vec<String> {
    let commit = repo.find_commit(commit_oid).unwrap();
    let tree = commit.tree().unwrap();
    tree.get_path(Path::new(filename))
        .ok()
        .and_then(|entry| repo.find_blob(entry.id()).ok())
        .map(|blob| {
            sanitize(decode(blob.content()))
                .lines()
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default()
}

// Retrieve the contents of a file from the working directory
pub fn get_file_at_workdir(repo: &Repository, filename: &str) -> Vec<String> {
    let full_path = repo
        .workdir()
        .map(|root| root.join(filename))
        .unwrap_or_else(|| Path::new(filename).to_path_buf());
    std::fs::read_to_string(full_path)
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}
