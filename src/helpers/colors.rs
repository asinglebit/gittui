#[rustfmt::skip]
use std::collections::HashMap;
use rand::seq::SliceRandom;
#[rustfmt::skip]
use ratatui::style::Color;
#[rustfmt::skip]
use crate::helpers::palette::*;

pub struct ColorPicker {
    lanes: HashMap<usize, bool>,
    palette_a: [Color; 8],
    palette_b: [Color; 8],
}

impl Default for ColorPicker {
    fn default() -> Self {
        ColorPicker {
            lanes: HashMap::new(),
            palette_a: [
                COLOR_GRASS,
                COLOR_LIME,
                COLOR_AMBER,
                COLOR_GRAPEFRUIT,
                COLOR_RED,
                COLOR_PURPLE,
                COLOR_INDIGO,
                COLOR_CYAN,
            ],
            palette_b: [
                COLOR_GREEN,
                COLOR_YELLOW,
                COLOR_ORANGE,
                COLOR_BROWN,
                COLOR_PINK,
                COLOR_DURPLE,
                COLOR_BLUE,
                COLOR_TEAL,
            ],
        }
    }
}

impl ColorPicker {
    pub fn alternate(&mut self, lane: usize) {
        self.lanes
            .entry(lane)
            .and_modify(|value| *value = !*value)
            .or_insert(false);
    }

    pub fn get(&self, lane: usize) -> Color {
        if self.lanes.get(&lane).copied().unwrap_or(false) {
            self.palette_b[lane % self.palette_b.len()]
        } else {
            self.palette_a[lane % self.palette_a.len()]
        }
    }
}

pub fn random_color() -> Color {
    let colors = [
        COLOR_PURPLE,
        COLOR_INDIGO,
        COLOR_CYAN,
        COLOR_GREEN,
        COLOR_LIME,
        COLOR_AMBER,
        COLOR_GRAPEFRUIT,
        COLOR_RED,
        COLOR_DURPLE,
        COLOR_BLUE,
        COLOR_TEAL,
        COLOR_GRASS,
        COLOR_YELLOW,
        COLOR_ORANGE,
        COLOR_BROWN,
        COLOR_PINK,
    ];

    // Pick a random one
    *colors.choose(&mut rand::thread_rng()).unwrap()
}
