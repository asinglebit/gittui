
#[rustfmt::skip]
use im::{
    Vector,
    OrdMap
};
#[rustfmt::skip]
use crate::core::chunk::{
    Chunk,
    NONE
};

#[derive(Default, Clone)]
pub struct Delta {
    pub ops: Vector<DeltaOp>,
}

#[derive(Clone)]
pub enum DeltaOp {
    Insert { index: usize, item: Chunk },
    Remove { index: usize },
    Replace { index: usize, new: Chunk },
}

#[derive(Default, Clone)]
pub struct Buffer {
    pub history: Vector<Vector<Chunk>>,
    pub curr: Vector<Chunk>,
    pub deltas: Vector<Delta>,
    pub checkpoints: OrdMap<usize, Vector<Chunk>>,
    pub delta: Delta,
    mergers: Vector<u32>,
}

impl Buffer {
    pub fn merger(&mut self, oidi: u32) {
        self.mergers.push_back(oidi);
    }

    pub fn update(&mut self, chunk: Chunk) {
        self.backup();  

        // Erase trailing dummy chunk
        while self.curr.last().is_some_and(|c| c.is_dummy()) {
            self.curr.pop_back();
            self.delta.ops.push_back(DeltaOp::Remove {
                index: self.curr.len() - 1,
            });
        }

        // let mut curr = self.curr.clone();

        // If we have a planned merge later on
        if let Some(merger_idx) = self.curr.iter().position(|inner| {
            self.mergers.iter().any(|oidi| *oidi == inner.oidi)
        }) {
            if let Some(merger_pos) = self.mergers.iter().position(|oidi| *oidi == self.curr[merger_idx].oidi) {
                self.mergers.remove(merger_pos);
            }

            let mut clone = self.curr[merger_idx].clone();
            clone.parent_a = clone.parent_b;
            clone.parent_b = NONE;
            self.curr[merger_idx].parent_b = NONE;
            self.curr.push_back(clone.clone());

            self.delta.ops.push_back(DeltaOp::Replace {
                index: merger_idx,
                new: self.curr[merger_idx].clone(),
            });

            self.delta.ops.push_back(DeltaOp::Insert {
                index: self.curr.len() - 1,
                item: clone,
            });
        }

        // Replace or append buffer chunk
        if let Some(first_idx) = self.curr.iter().position(|inner| {
            inner.parent_a == chunk.oidi
        }) {
            let old_oidi = chunk.oidi;

            // Replace chunk
            self.curr[first_idx] = chunk.clone();
            self.delta.ops.push_back(DeltaOp::Replace {
                index: first_idx,
                new: chunk,
            });

            // Place dummies in case of branching
            for (i, inner) in self.curr.iter_mut().enumerate() {
                if inner.oidi == old_oidi {
                    continue;
                }

                let mut parents_changed = false;

                if inner.parent_a == old_oidi {
                    inner.parent_a = NONE;
                    parents_changed = true;
                }

                if inner.parent_b == old_oidi {
                    inner.parent_b = NONE;
                    parents_changed = true;
                }

                if parents_changed && inner.parent_a == NONE && inner.parent_b== NONE {
                    *inner = Chunk::dummy();
                    self.delta.ops.push_back(DeltaOp::Replace {
                        index: i,
                        new: inner.clone(),
                    });
                } else {
                    self.delta.ops.push_back(DeltaOp::Replace {
                        index: i,
                        new: inner.clone(),
                    });
                }
            }
        } else {
            self.curr.push_back(chunk.clone());
            self.delta.ops.push_back(DeltaOp::Insert {
                index: self.curr.len() - 1,
                item: chunk,
            });
        }

    }

    pub fn backup(&mut self) {
        let old = std::mem::take(&mut self.delta);
        self.deltas.push_back(old);
        let idx = self.deltas.len().saturating_sub(1);
        if  idx % 100 == 0 {
            self.checkpoints.insert(idx, self.curr.clone());
        }        
    }

    pub fn decompress(&mut self, start: usize, end: usize) {
        self.history.clear();

        // Find nearest checkpoint, rewrite this later to binary search
        let checkpoint_idx = self.checkpoints.keys()
            .rev()
            .find(|&&idx| idx <= start)
            .copied();
        
        // Start from the checkpoint snapshot, or empty
        let mut curr = checkpoint_idx
            .and_then(|idx| self.checkpoints.get(&idx))
            .cloned()
            .unwrap_or_default();
        
        // Determine the first delta to apply
        let begin = checkpoint_idx.map_or(0, |idx| idx + 1);
        let end = end.min(self.deltas.len());
        
        for delta in self.deltas.iter().skip(begin).take(end - begin) {
            for op in delta.ops.iter() {
                match op {
                    DeltaOp::Insert { index, item } => {
                        curr.insert(*index, item.clone());
                    }
                    DeltaOp::Remove { index } => {
                        curr.remove(*index);
                    }
                    DeltaOp::Replace { index, new } => {
                        curr[*index] = new.clone();
                    }
                }
            }
            self.history.push_back(curr.clone());
        }
    }
}
