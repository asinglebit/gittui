#[rustfmt::skip]
use std::{
    collections::{
        HashSet,
        HashMap
    },
    path::Path
};
#[rustfmt::skip]
use git2::{
    DiffOptions,
    ObjectType,
    BranchType,
    Delta,
    Oid,
    Repository,
    StatusOptions,
    Time
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

        // Skip files that are committed (no changes)
        if status.is_empty() {
            continue;
        }

        // Check staged changes (INDEX vs HEAD)
        if status.is_index_modified() {
            changes.staged.modified.push(path.clone());
        }
        if status.is_index_new() {
            changes.staged.added.push(path.clone());
        }
        if status.is_index_deleted() {
            changes.staged.deleted.push(path.clone());
        }

        // Check unstaged changes (WORKDIR vs INDEX)
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

    // Deduplicate filenames across staged + unstaged
    let modified: HashSet<_> = changes
        .staged
        .modified
        .iter()
        .chain(&changes.unstaged.modified)
        .cloned()
        .collect();

    let added: HashSet<_> = changes
        .staged
        .added
        .iter()
        .chain(&changes.unstaged.added)
        .cloned()
        .collect();

    let deleted: HashSet<_> = changes
        .staged
        .deleted
        .iter()
        .chain(&changes.unstaged.deleted)
        .cloned()
        .collect();

    // Counts after deduplication
    changes.modified_count = modified.len();
    changes.added_count = added.len();
    changes.deleted_count = deleted.len();
    changes.is_staged = changes.staged.modified.len() > 0
        || changes.staged.added.len() > 0
        || changes.staged.deleted.len() > 0;
    changes.is_unstaged = changes.unstaged.modified.len() > 0
        || changes.unstaged.added.len() > 0
        || changes.unstaged.deleted.len() > 0;
    changes.is_clean = !changes.is_staged && !changes.is_unstaged;

    Ok(changes)
}

pub fn get_changed_filenames(repo: &Repository, oid: Oid) -> Vec<FileChange> {
    let commit = repo.find_commit(oid).unwrap();
    let tree = commit.tree().unwrap();
    let mut changes = Vec::new();

    // Helper to recursively walk a tree and collect all files
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
                            status: FileStatus::Added, // initial commit / folder contents
                        });
                    }
                    Some(ObjectType::Tree) => {
                        // Recurse into subdirectory
                        if let Ok(subtree) = entry.to_object(repo).and_then(|o| o.peel_to_tree()) {
                            walk_tree(repo, &subtree, &path, changes);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Initial commit: recursively collect all files
    if commit.parent_count() == 0 {
        walk_tree(repo, &tree, "", &mut changes);
        return changes;
    }

    // Normal commit: diff against parent
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
            .unwrap();

        // If the delta points to a folder (no extension, tree object), expand recursively
        let path_str = path.display().to_string();

        // TODO: think of something better later.
        // We want to make sure we only recurse through folders but we want it to be cheap
        // Crude check: no '.' -> folder
        let is_folder = !path_str.contains('.');

        if is_folder {
            if let Ok(tree_obj) = repo.find_tree(delta.new_file().id()) {
                walk_tree(repo, &tree_obj, &path_str, &mut changes);
                continue;
            }
        }

        let status = match delta.status() {
            Delta::Added => FileStatus::Added,
            Delta::Modified => FileStatus::Modified,
            Delta::Deleted => FileStatus::Deleted,
            Delta::Renamed => FileStatus::Renamed,
            _ => FileStatus::Other,
        };

        changes.push(FileChange {
            filename: path_str,
            status,
        });
    }

    changes
}

#[derive(Debug)]
pub struct LineChange {
    pub origin: char,    // '+', '-', ' ' for added, removed, context
    pub content: String, // line content
}

#[derive(Debug)]
pub struct Hunk {
    pub header: String, // hunk header, e.g. @@ -1,3 +1,4 @@
    pub lines: Vec<LineChange>,
}

// Get the changes (hunks + lines) for a single file in a specific commit
pub fn get_file_diff(
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

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(filename);

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

    let mut hunks_result = Vec::new();

    // Use diff.print() to iterate hunks and lines sequentially
    diff.print(git2::DiffFormat::Patch, |delta, hunk_opt, line| {
        // Only create a new Hunk when hunk_opt is Some
        if let Some(hunk) = hunk_opt {
            hunks_result.push(Hunk {
                header: String::from_utf8_lossy(hunk.header()).to_string(),
                lines: Vec::new(),
            });
        }

        // Add line to the last hunk
        if let Some(last_hunk) = hunks_result.last_mut() {
            last_hunk.lines.push(LineChange {
                origin: line.origin() as char,
                content: String::from_utf8_lossy(line.content()).to_string(),
            });
        }

        true
    })?;

    Ok(hunks_result)
}

// Get the original file lines from the commit
pub fn get_file_lines_at_commit(repo: &Repository, commit_oid: Oid, filename: &str) -> Vec<String> {
    let commit = repo.find_commit(commit_oid).unwrap();
    let tree = commit.tree().unwrap();

    if let Ok(entry) = tree.get_path(Path::new(filename)) {
        if let Ok(blob) = repo.find_blob(entry.id()) {
            return String::from_utf8_lossy(blob.content())
                .lines()
                .map(|s| s.to_string())
                .collect();
        }
    }

    Vec::new()
}
