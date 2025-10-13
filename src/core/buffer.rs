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
            .position(|inner| self.mergers.iter().any(|oid| oid == &inner.oid))
        {
            // Find the index in `self.mergers` of the matching SHA
            if let Some(merger_pos) = self
                .mergers
                .iter()
                .position(|oid| oid == &self.curr[merger_idx].oid)
            {
                self.mergers.remove(merger_pos);
            }

            // Clone the element at merger_idx
            let mut clone = self.curr[merger_idx].clone();
            clone.parents.remove(0);

            // Remove second parent from the original
            self.curr[merger_idx].parents.remove(1);

            // Insert it right after the found index
            self.curr.push(clone);
        }

        // Replace or append buffer metadata
        if let Some(first_idx) = self
            .curr
            .iter()
            .position(|inner| inner.parents.contains(&metadata.oid))
        {
            let old_oid = metadata.oid;

            // Replace metadata
            self.curr[first_idx] = metadata;
            let keep_ptr = self.curr[first_idx].parents.as_ptr();

            // Place dummies in case of branching
            for inner in self.curr.iter_mut() {
                if inner.parents.contains(&old_oid) && inner.parents.as_ptr() != keep_ptr {
                    if inner.parents.len() > 1 {
                        inner.parents.retain(|oid| *oid != old_oid);
                    } else {
                        *inner = Chunk::dummy();
                    }
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
