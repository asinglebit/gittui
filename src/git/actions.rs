use git2::ObjectType;
#[rustfmt::skip]
use git2::{
    Oid,
    Error,
    ErrorCode,
    Signature,
    StatusOptions,
    BranchType,
    ResetType,
    Repository,
    build::CheckoutBuilder
};

pub fn checkout_head(repo: &Repository, oid: Oid) {
    // Find the commit object
    let commit = repo.find_commit(oid).unwrap();

    // Set HEAD to the commit (detached)
    repo.set_head_detached(commit.id()).unwrap();

    // Checkout the commit
    repo.checkout_head(Some(
        CheckoutBuilder::default().allow_conflicts(true).force(), // optional: force overwrite local changes
    ))
    .expect("Error checking out");
}

pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    let local_branch_name = branch_name.strip_prefix("origin/").unwrap_or(branch_name);

    // Always try the local branch name first
    match repo.find_branch(local_branch_name, BranchType::Local) {
        Ok(branch) => {
            // Switch to existing local branch
            repo.set_head(branch.get().name().unwrap())?;
        }
        Err(_) => {
            // Create new local branch from remote
            let remote_branch_name = if branch_name.starts_with("origin/") {
                branch_name
            } else {
                &format!("origin/{}", branch_name)
            };

            let remote_branch = repo.find_branch(remote_branch_name, BranchType::Remote)?;
            let commit = remote_branch.get().peel_to_commit()?;

            let mut local_branch = repo.branch(local_branch_name, &commit, false)?;
            local_branch.set_upstream(Some(remote_branch_name))?;

            repo.set_head(local_branch.get().name().unwrap())?;
        }
    }

    repo.checkout_head(Some(
        CheckoutBuilder::default().allow_conflicts(true).force(),
    ))?;

    Ok(())
}

pub fn git_add_all(repo: &Repository) -> Result<(), Error> {
    let mut index = repo.index()?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .include_unmodified(false);

    let statuses = repo.statuses(Some(&mut opts))?;

    for entry in statuses.iter() {
        if let Some(path) = entry.path() {
            let path = std::path::Path::new(path);

            match entry.status() {
                s if s.is_wt_deleted() || s.is_index_deleted() => {
                    // Stage deletions (whether from working dir or already staged)
                    if index.get_path(path, 0).is_some() {
                        index.remove_path(path)?;
                    }
                }
                _ => {
                    // Stage new or modified files
                    index.add_path(path)?;
                }
            }
        }
    }

    index.write()?;
    Ok(())
}

pub fn commit_staged(
    repo: &Repository,
    message: &str,
    name: &str,
    email: &str,
) -> Result<Oid, Error> {
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Determine parent commit
    let parent_commit = match repo.head() {
        Ok(head_ref) => {
            // Try to peel to commit
            match head_ref.peel_to_commit() {
                Ok(commit) => Some(commit),
                Err(_) => None, // HEAD exists but not pointing to a commit
            }
        }
        Err(e) => {
            if e.code() == ErrorCode::UnbornBranch {
                None // empty repo, initial commit
            } else {
                return Err(e);
            }
        }
    };

    let signature = Signature::now(name, email)?;

    let commit_oid = if let Some(parent) = parent_commit {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent],
        )?
    } else {
        // Initial commit
        repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?
    };

    Ok(commit_oid)
}

pub fn reset_to_commit(repo: &Repository, target: Oid, reset_type: ResetType) -> Result<(), Error> {
    // Resolve the target commit object
    let target_commit = repo.find_commit(target)?;

    // Get HEAD reference
    let head = repo.head()?;

    if head.is_branch() {
        // Normal branch: move branch reference
        let branch_ref_name = head
            .name()
            .ok_or_else(|| Error::from_str("Invalid branch reference name"))?;
        let mut branch_ref = repo.find_reference(branch_ref_name)?;
        branch_ref.set_target(target, "reset branch to commit")?;
    } else {
        // Detached HEAD: move HEAD directly
        let head_ref_name = head.name().unwrap_or("HEAD");
        let mut head_ref_obj = repo.find_reference(head_ref_name)?;
        head_ref_obj.set_target(target, "reset detached HEAD")?;
    }

    // Perform the reset (Hard, Soft, or Mixed)
    repo.reset(&target_commit.into_object(), reset_type, None)?;

    Ok(())
}
