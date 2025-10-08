use git2::Oid;

#[derive(Clone)]
pub struct Chunk {
    pub sha: Oid,
    pub parents: Vec<Oid>,
}

impl Chunk {
    pub fn dummy() -> Self {
        Chunk {
            sha: Oid::zero(),
            parents: Vec::new(),
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.sha == Oid::zero() && self.parents.is_empty()
    }
}
