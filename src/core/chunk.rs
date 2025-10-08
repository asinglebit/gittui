#[rustfmt::skip]
use git2::Oid;

#[derive(Clone, PartialEq)]
pub enum Markers {
    Uncommitted,
    Commit,
    Dummy,
}

#[derive(Clone)]
pub struct Chunk {
    pub oid: Oid,
    pub parents: Vec<Oid>,
    pub marker: Markers,
}

impl Chunk {
    pub fn uncommitted(parents: Vec<Oid>) -> Self {
        Chunk {
            oid: Oid::zero(),
            parents,
            marker: Markers::Uncommitted,
        }
    }

    pub fn commit(oid: Oid, parents: Vec<Oid>) -> Self {
        Chunk {
            oid,
            parents,
            marker: Markers::Commit,
        }
    }

    pub fn dummy() -> Self {
        Chunk {
            oid: Oid::zero(),
            parents: Vec::new(),
            marker: Markers::Dummy,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.marker == Markers::Dummy
    }
}
