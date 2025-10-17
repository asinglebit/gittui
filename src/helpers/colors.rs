#[rustfmt::skip]
use std::collections::HashMap;
#[rustfmt::skip]
use ratatui::style::Color;
#[rustfmt::skip]
use crate::helpers::palette::*;

#[derive(Clone)]
pub struct ColorPicker {
    lanes: HashMap<usize, bool>,
    palette_a: [Color; 16],
    palette_b: [Color; 8],
}

impl Default for ColorPicker {
    fn default() -> Self {
        ColorPicker {
            lanes: HashMap::new(),
            palette_a: [
                COLOR_GRASS,
                COLOR_GREEN,
                COLOR_CYAN,
                COLOR_TEAL,
                COLOR_INDIGO,
                COLOR_BLUE,
                COLOR_PURPLE,
                COLOR_DURPLE,
                COLOR_RED,
                COLOR_PINK,
                COLOR_GRAPEFRUIT,
                COLOR_BROWN,
                COLOR_AMBER,
                COLOR_ORANGE,
                COLOR_LIME,
                COLOR_YELLOW,
            ],
            palette_b: [
                COLOR_GREEN,
                COLOR_TEAL,
                COLOR_BLUE,
                COLOR_DURPLE,
                COLOR_PINK,
                COLOR_BROWN,
                COLOR_ORANGE,
                COLOR_YELLOW,
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
