
#[rustfmt::skip]
use im::{
    Vector,
    OrdMap
};
#[rustfmt::skip]
use git2::Oid;
#[rustfmt::skip]
use crate::core::chunk::Chunk;

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
            self.delta.ops.push_back(DeltaOp::Remove {
                index: self.curr.len() - 1,
            });
        }

        let mut curr = self.curr.clone();

        // If we have a planned merge later on
        if let Some(merger_idx) = curr.iter().position(|inner| {
            self.mergers.iter().any(|oid| Some(oid) == inner.oid.as_ref())
        }) {
            if let Some(merger_pos) = self.mergers.iter().position(|oid| Some(oid) == curr[merger_idx].oid.as_ref()) {
                self.mergers.remove(merger_pos);
            }

            let mut clone = curr[merger_idx].clone();
            clone.parent_a = clone.parent_b;
            clone.parent_b = None;
            curr[merger_idx].parent_b = None;
            curr.push_back(clone.clone());

            self.delta.ops.push_back(DeltaOp::Replace {
                index: merger_idx,
                new: curr[merger_idx].clone(),
            });

            self.delta.ops.push_back(DeltaOp::Insert {
                index: curr.len() - 1,
                item: clone,
            });
        }

        // Replace or append buffer metadata
        if let Some(first_idx) = curr.iter().position(|inner| {
            inner.parent_a.as_ref() == metadata.oid.as_ref()
            // inner.parent_b.as_ref() == metadata.oid.as_ref()
        }) {
            let old_oid = metadata.oid;

            // Replace metadata
            curr[first_idx] = metadata.clone();
            self.delta.ops.push_back(DeltaOp::Replace {
                index: first_idx,
                new: metadata,
            });

            // Place dummies in case of branching
            curr = curr
                .into_iter()
                .enumerate()
                .map(|(i, mut inner)| {
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
                        self.delta.ops.push_back(DeltaOp::Replace {
                            index: i,
                            new: Chunk::dummy(),
                        });
                        Chunk::dummy()
                    } else {
                        self.delta.ops.push_back(DeltaOp::Replace {
                            index: i,
                            new: inner.clone(),
                        });
                        inner
                    }
                })
                .collect();
        } else {
            curr.push_back(metadata.clone());
            self.delta.ops.push_back(DeltaOp::Insert {
                index: curr.len() - 1,
                item: metadata,
            });
        }

        self.curr = curr;
    }

    pub fn backup(&mut self) {
        self.deltas.push_back(self.delta.clone());
        self.delta = Delta::default();
        
        let idx = self.deltas.len().saturating_sub(1);
        if  idx % 500 == 0 {
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
