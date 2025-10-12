#[rustfmt::skip]
use std::{
    sync::{
        Arc,
        Mutex
    },
};
#[rustfmt::skip]
use git2::{
    BranchType,
    Oid,
    Repository,
    Revwalk,
};
pub struct LazyWalker {
    revwalk: Mutex<Revwalk<'static>>,
}

impl LazyWalker {
    
    pub fn new(repo: Arc<Repository>) -> Result<Self, git2::Error> {
        let revwalk = Self::build_revwalk(&repo)?;
        Ok(Self {
            revwalk: Mutex::new(revwalk),
        })
    }

    pub fn reset(&self, repo: Arc<Repository>) -> Result<(), git2::Error> {
        let revwalk = Self::build_revwalk(&repo)?;
        let mut guard = self.revwalk.lock().unwrap();
        *guard = revwalk;
        Ok(())
    }

    // Get up to `count` commits from the global revwalk
    pub fn next_chunk(&self, count: usize) -> Vec<Oid> {
        let mut revwalk = self.revwalk.lock().unwrap();
        revwalk.by_ref().take(count).filter_map(Result::ok).collect()
    }
    
    fn build_revwalk(repo: &Repository) -> Result<Revwalk<'static>, git2::Error> {
        // SAFETY: we keep repo alive in Arc, so transmute to 'static is safe
        let repo_ref: &'static Repository =
            unsafe { std::mem::transmute::<&Repository, &'static Repository>(repo) };

        let mut revwalk = repo_ref.revwalk()?;

        // Push all local and remote branch tips
        for branch_type in [BranchType::Local, BranchType::Remote] {
            for branch in repo.branches(Some(branch_type))? {
                let (branch, _) = branch?;
                if let Some(oid) = branch.get().target() {
                    revwalk.push(oid)?;
                }
            }
        }

        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        Ok(revwalk)
    }
}
