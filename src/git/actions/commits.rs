#[rustfmt::skip]
use std::{
    thread,
    collections::{
        HashMap
    }
};
#[rustfmt::skip]
use git2::{
    Oid,
    Cred,
    RemoteCallbacks,
    Error,
    ErrorCode,
    Signature,
    StatusOptions,
    BranchType,
    ResetType,
    Repository,
    FetchOptions,
    PushOptions,
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

pub fn checkout_branch(
    repo: &Repository,
    visible_branches: &mut HashMap<Oid, Vec<String>>,
    oid: Oid,
    branch_name: &str,
) -> Result<(), git2::Error> {
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

            visible_branches
                .entry(oid)
                .or_insert_with(Vec::new)
                .push(local_branch_name.to_string());

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
            head_ref.peel_to_commit().ok()
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

pub fn unstage_all(repo: &Repository) -> Result<(), git2::Error> {
    // Get HEAD commit
    let head = match repo.head() {
        Ok(head) => head.peel_to_commit()?,
        Err(_) => {
            // If no HEAD exists (fresh repo), there's nothing to unstage
            return Ok(());
        }
    };

    // Perform mixed reset - keeps working directory changes but resets index to HEAD
    repo.reset(&head.into_object(), ResetType::Mixed, None)?;

    Ok(())
}

pub fn fetch_over_ssh(
    repo_path: &str,
    remote_name: &str,
) -> thread::JoinHandle<Result<(), git2::Error>> {
    // Clone the strings so the thread owns them
    let repo_path = repo_path.to_string();
    let remote_name = remote_name.to_string();

    thread::spawn(move || {
        let repo = Repository::open(repo_path)?;
        let mut remote = repo.find_remote(&remote_name)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _| {
            Cred::ssh_key_from_agent(username_from_url.unwrap())
        });

        callbacks.transfer_progress(|_stats| {
            // println!("Received {}/{} objects", stats.received_objects(), stats.total_objects());
            true
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )?;
        Ok(())
    })
}

pub fn push_over_ssh(
    repo_path: &str,
    remote_name: &str,
    branch: &str,
    force: bool,
) -> thread::JoinHandle<Result<(), git2::Error>> {
    // Clone inputs so they can move into the thread safely
    let repo_path = repo_path.to_string();
    let remote_name = remote_name.to_string();
    let branch = branch.to_string();

    thread::spawn(move || {
        // Open the repository
        let repo = Repository::open(&repo_path)?;
        let mut remote = repo.find_remote(&remote_name)?;

        // Configure SSH authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _| Cred::ssh_key_from_agent("git"));

        // Track progress
        callbacks.push_update_reference(|_refname, status| {
            if let Some(_err) = status {
                // eprintln!("Failed to update {refname}: {err}");
            } else {
                // println!("Updated {refname}");
            }
            Ok(())
        });

        // Configure push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // The refspec tells Git what to push
        let refspec = if force {
            format!("+refs/heads/{0}:refs/heads/{0}", branch) // '+' means force
        } else {
            format!("refs/heads/{0}:refs/heads/{0}", branch)
        };

        // Perform the push
        remote.push(&[&refspec], Some(&mut push_options))?;

        // println!("Push complete for branch '{}'", branch);
        Ok(())
    })
}
