use git2::{Oid, Repository, build::CheckoutBuilder};

pub fn checkout(repo: &Repository, oid: Oid) {
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
