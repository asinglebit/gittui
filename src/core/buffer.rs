#[rustfmt::skip]
use git2::Oid;
#[rustfmt::skip]
use crate::core::chunk::Chunk;

#[derive(Default, Clone)]
pub struct Buffer {
    pub prev: Vec<Chunk>,
    pub curr: Vec<Chunk>,
    mergers: Vec<Oid>,
}

impl Buffer {
    pub fn merger(&mut self, oid: Oid) {
        self.mergers.push(oid);
    }

    pub fn update(&mut self, metadata: Chunk) {
        // Erase trailing dummy metadata
        while self.curr.last().is_some_and(|c| c.is_dummy()) {
            self.curr.pop();
        }

        // If we have a planned merge later on
        if let Some(merger_idx) = self
            .curr
            .iter()
            .position(|inner| self.mergers.iter().any(|oid| Some(oid) == inner.oid.as_ref()))
        {
            // Find the index in `self.mergers` of the matching SHA
            if let Some(merger_pos) = self
                .mergers
                .iter()
                .position(|oid| Some(oid) == self.curr[merger_idx].oid.as_ref())
            {
                self.mergers.remove(merger_pos);
            }

            // Clone the element at merger_idx
            let mut clone = self.curr[merger_idx].clone();
            clone.parent_a = None;

            // Remove second parent from the original
            self.curr[merger_idx].parent_b = None;

            // Insert it right after the found index
            self.curr.push(clone);
        }

        // Replace or append buffer metadata
        if let Some(first_idx) = self
            .curr
            .iter()
            .position(|inner| 
                inner.parent_a.as_ref() == metadata.oid.as_ref() ||
                inner.parent_b.as_ref() == metadata.oid.as_ref()
            )
        {
            let old_oid = metadata.oid;

            // Replace metadata
            self.curr[first_idx] = metadata;

            // Place dummies in case of branching
            for inner in self.curr.iter_mut() {
                // Skip the newly replaced one
                if inner.oid == old_oid {
                    continue;
                }

                let mut parents_changed = false;

                // Remove old_oid from parent_a
                if inner.parent_a.as_ref() == old_oid.as_ref() {
                    inner.parent_a = None;
                    parents_changed = true;
                }

                // Remove old_oid from parent_b
                if inner.parent_b.as_ref() == old_oid.as_ref() {
                    inner.parent_b = None;
                    parents_changed = true;
                }

                // If both parents are None, replace with dummy
                if parents_changed && inner.parent_a.is_none() && inner.parent_b.is_none() {
                    *inner = Chunk::dummy();
                }
            }
        } else {
            self.curr.push(metadata);
        }
    }

    pub fn backup(&mut self) {
        self.prev = self.curr.clone();
    }
}
