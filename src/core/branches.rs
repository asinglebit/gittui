#[rustfmt::skip]
use std::{
    cell::{
        RefCell
    },
    rc::Rc,
    collections::{
        HashMap
    },
};
#[rustfmt::skip]
use ratatui::{
    style::{
        Color
    }
};
#[rustfmt::skip]
use crate::{
    core::{
        oids::{
            Oids
        }
    },
    helpers::{
        colors::{
            ColorPicker
        }
    }
};

#[derive(Default, Clone)]
pub struct Branches {
    pub local: HashMap<u32, Vec<String>>,
    pub remote: HashMap<u32, Vec<String>>,
    pub all: HashMap<u32, Vec<String>>,
    pub colors: HashMap<u32, Color>,
    pub sorted: Vec<(u32, String)>,
    pub indices: Vec<usize>,
    pub visible: HashMap<u32, Vec<String>>,
}

impl Branches {
    pub fn feed(
        &mut self,
        oids: &Oids,
        color: &Rc<RefCell<ColorPicker>>,
        branches_lanes: &HashMap<u32, usize>,
        branches_local: HashMap<u32, Vec<String>>,
        branches_remote: HashMap<u32, Vec<String>>
    ) {
        // Initialize
        self.local = branches_local;
        self.remote = branches_remote;
        self.all = HashMap::new();
        self.colors = HashMap::new();
        self.sorted = Vec::new();
        self.indices = Vec::new();
        
        // Combine local and remote branches
        for (&alias, branches) in self.local.iter() {
            self.all.insert(alias, branches.clone());
        }
        for (&oidi, branches) in self.remote.iter() {
            self.all
                .entry(oidi)
                .and_modify(|existing| existing.extend(branches.iter().cloned()))
                .or_insert_with(|| branches.clone());
        }

        // Make all branches visible if none are
        if self.visible.is_empty() {
            for (&alias, branches) in self.all.iter() {
                self.visible.insert(alias, branches.clone());
            }
        }
        
        // Branch tuple vectors
        let mut local: Vec<(u32, String)> = self.local.iter().flat_map(|(&alias, branches)| {
                branches.iter().map(move |branch| (alias, branch.clone()))
            }).collect();
        let mut remote: Vec<(u32, String)> = self.remote.iter().flat_map(|(&alias, branches)| {
                branches.iter().map(move |branch| (alias, branch.clone()))
            }).collect();

        // Sorting tuples
        local.sort_by(|a, b| a.1.cmp(&b.1));
        remote.sort_by(|a, b| a.1.cmp(&b.1));

        // Combining into sorted
        self.sorted = local.into_iter().chain(remote).collect();

        // Set branch colors
        for (oidi, &lane_idx) in branches_lanes.iter() {
            self.colors.insert(*oidi, color.borrow().get_lane(lane_idx));
        }
        
        // Build a lookup of branch aliases to positions in sorted aliases
        let mut sorted_time = self.sorted.clone();
        let index_map: std::collections::HashMap<u32, usize> = oids.get_sorted_aliases().iter().enumerate().map(|(i, &oidi)| (oidi, i)).collect();

        // Sort the vector using the index map
        sorted_time.sort_by_key(|(oidi, _)| index_map.get(oidi).copied().unwrap_or(usize::MAX));
        self.indices = Vec::new();
        sorted_time.iter().for_each(|(oidi, _)| {
            self.indices.push(oids.get_sorted_aliases().iter().position(|o| oidi == o).unwrap_or(usize::MAX));
        });
    }
}
