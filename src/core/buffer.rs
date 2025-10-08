use git2::Oid;

use crate::core::chunk::Chunk;

pub struct Buffer {
    pub prev: Vec<Chunk>,
    pub curr: Vec<Chunk>,
    mergers: Vec<Oid>,
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            prev: Vec::new(),
            curr: Vec::new(),
            mergers: Vec::new(),
        }
    }
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
            .position(|inner| self.mergers.iter().any(|sha| sha == &inner.sha))
        {
            // Find the index in `self.mergers` of the matching SHA
            if let Some(merger_pos) = self
                .mergers
                .iter()
                .position(|sha| sha == &self.curr[merger_idx].sha)
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
            .position(|inner| inner.parents.contains(&metadata.sha))
        {
            let old_sha = metadata.sha;

            // Replace metadata
            self.curr[first_idx] = metadata;
            let keep_ptr = self.curr[first_idx].parents.as_ptr();

            // Place dummies in case of branching
            for inner in self.curr.iter_mut() {
                if inner.parents.contains(&old_sha) && inner.parents.as_ptr() != keep_ptr {
                    if inner.parents.len() > 1 {
                        inner.parents.retain(|sha| *sha != old_sha);
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
