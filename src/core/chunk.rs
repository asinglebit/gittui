use git2::Oid;

#[derive(Clone, PartialEq)]
pub enum Markers {
    Uncommitted,
    Commit,
    Dummy
}

#[derive(Clone)]
pub struct Chunk {
    pub sha: Oid,
    pub parents: Vec<Oid>,
    pub marker: Markers
}

impl Chunk {

    pub fn uncommitted(parents: Vec<Oid>) -> Self {
        Chunk {
            sha: Oid::zero(),
            parents,
            marker: Markers::Uncommitted
        }
    }

    pub fn commit(sha: Oid, parents: Vec<Oid>) -> Self {
        Chunk {
            sha,
            parents,
            marker: Markers::Commit
        }
    }

    pub fn dummy() -> Self {
        Chunk {
            sha: Oid::zero(),
            parents: Vec::new(),
            marker: Markers::Dummy
        }
    }

    pub fn is_uncommitted(&self) -> bool {
        self.marker == Markers::Uncommitted
    }

    pub fn is_commit(&self) -> bool {
        self.marker == Markers::Commit
    }

    pub fn is_dummy(&self) -> bool {
        self.marker == Markers::Dummy
    }
}
