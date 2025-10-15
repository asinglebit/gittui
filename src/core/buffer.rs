#[rustfmt::skip]
use im::Vector;
#[rustfmt::skip]
use git2::Oid;
#[rustfmt::skip]
use crate::core::chunk::Chunk;

#[derive(Default, Clone)]
pub struct Buffer {
    pub history: Vector<Vector<Chunk>>,
    pub curr: Vector<Chunk>,
    mergers: Vector<Oid>,
}

impl Buffer {
    pub fn merger(&mut self, oid: Oid) {
        self.mergers.push_back(oid);
    }

    pub fn update(&mut self, metadata: Chunk) {
        self.backup();

        // Erase trailing dummy metadata
        while self.curr.last().is_some_and(|c| c.is_dummy()) {
            self.curr.pop_back();
        }

        let mut curr = self.curr.clone();

        // If we have a planned merge later on
        if let Some(merger_idx) = curr.iter().position(|inner| {
            self.mergers
                .iter()
                .any(|oid| Some(oid) == inner.oid.as_ref())
        }) {
            if let Some(merger_pos) = self
                .mergers
                .iter()
                .position(|oid| Some(oid) == curr[merger_idx].oid.as_ref())
            {
                self.mergers.remove(merger_pos);
            }

            let mut clone = curr[merger_idx].clone();
            clone.parent_a = None;

            curr[merger_idx].parent_b = None;
            curr.push_back(clone);
        }

        // Replace or append buffer metadata
        if let Some(first_idx) = curr.iter().position(|inner| {
            inner.parent_a.as_ref() == metadata.oid.as_ref()
                || inner.parent_b.as_ref() == metadata.oid.as_ref()
        }) {
            let old_oid = metadata.oid;

            // Replace metadata
            curr[first_idx] = metadata;

            // Place dummies in case of branching
            curr = curr
                .iter()
                .cloned()
                .map(|mut inner| {
                    if inner.oid == old_oid {
                        return inner;
                    }

                    let mut parents_changed = false;

                    if inner.parent_a.as_ref() == old_oid.as_ref() {
                        inner.parent_a = None;
                        parents_changed = true;
                    }

                    if inner.parent_b.as_ref() == old_oid.as_ref() {
                        inner.parent_b = None;
                        parents_changed = true;
                    }

                    if parents_changed && inner.parent_a.is_none() && inner.parent_b.is_none() {
                        Chunk::dummy()
                    } else {
                        inner
                    }
                })
                .collect();
        } else {
            curr.push_back(metadata);
        }

        self.curr = curr;
    }

    pub fn backup(&mut self) {
        
        // Append immutable snapshot to history
        self.history.push_back(self.curr.clone());
    }
}
