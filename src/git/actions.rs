#[rustfmt::skip]
use git2::{
    Oid,
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

pub fn reset_hard(repo: &Repository, target: &str) -> Result<(), git2::Error> {
    let obj = repo.revparse_single(target)?;
    repo.reset(&obj, ResetType::Hard, None)?;
    Ok(())
}
