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
    pub oid: Option<Oid>,
    pub parent_a: Option<Oid>,
    pub parent_b: Option<Oid>,
    pub marker: Markers,
}

impl Chunk {
    pub fn uncommitted(parent_a: Option<Oid>, parent_b: Option<Oid>) -> Self {
        Chunk {
            oid: None,
            parent_a,
            parent_b,
            marker: Markers::Uncommitted,
        }
    }

    pub fn commit(oid: Option<Oid>, parent_a: Option<Oid>, parent_b: Option<Oid>) -> Self {
        Chunk {
            oid,
            parent_a,
            parent_b,
            marker: Markers::Commit,
        }
    }

    pub fn dummy() -> Self {
        Chunk {
            oid: None,
            parent_a: None,
            parent_b: None,
            marker: Markers::Dummy,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.marker == Markers::Dummy
    }
}
