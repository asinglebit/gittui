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
        palette::{
            Theme
        },
        colors::{
            ColorPicker
        }
    }
};

pub struct Tags {
    pub local: HashMap<u32, Vec<String>>,
    pub colors: HashMap<u32, Color>,
    pub sorted: Vec<(u32, String)>,
    pub indices: Vec<usize>,
    pub visible: bool
}

impl Default for Tags {

    fn default() -> Self {
        Self {
            local: Default::default(),
            colors: Default::default(),
            sorted: Default::default(),
            indices: Default::default(),
            visible: Default::default(),
        }
    }
}

impl Tags {

    pub fn feed(
        &mut self,
        oids: &Oids,
        color: &Rc<RefCell<ColorPicker>>,
        tags_lanes: &HashMap<u32, usize>,
        tags_local: HashMap<u32, Vec<String>>,
    ) {

        // Initialize
        self.local = tags_local;
        self.colors = HashMap::new();
        self.sorted = Vec::new();
        self.indices = Vec::new();
        
        // Branch tuple vectors
        let mut sorted: Vec<(u32, String)> = self.local.iter().flat_map(|(&alias, tags)| {
                tags.iter().map(move |tag| (alias, tag.clone()))
            }).collect();

        // Sorting tuples
        sorted.sort_by(|a, b| a.1.cmp(&b.1));

        // Set tag colors
        for (oidi, &lane_idx) in tags_lanes.iter() {
            self.colors.insert(*oidi, color.borrow().get_lane(lane_idx));
        }
        
        // Build a lookup of tag aliases to positions in sorted aliases
        let mut sorted_time = self.sorted.clone();
        let index_map: std::collections::HashMap<u32, usize> = oids.get_sorted_aliases().iter().enumerate().map(|(i, &oidi)| (oidi, i)).collect();
        // Sort the vector using the index map

        sorted_time.sort_by_key(|(oidi, _)| index_map.get(oidi).copied().unwrap_or(usize::MAX));
        self.indices = Vec::new();
        sorted_time.iter().for_each(|(oidi, _)| {
            self.indices.push(oids.get_sorted_aliases().iter().position(|o| oidi == o).unwrap_or(usize::MAX));
        });
    }

    pub fn get_sorted_aliases(&self) -> &Vec<(u32, String)> {
        &self.sorted
    }

    pub fn get_color(&self, theme: &Theme, tag_alias: &u32) -> Color {
        *self.colors.get(tag_alias).unwrap_or(&theme.COLOR_TEXT)
    }

    pub fn is_local(&self, tag_name: &String) -> bool {
        self.local
            .values()
            .any(|tags| tags.iter().any(|current_tag| current_tag.as_str() == tag_name))
    }

}
