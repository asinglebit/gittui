use std::{cell::RefCell, collections::HashMap};

use ratatui::{
    style::{Color, Style},
    text::Span,
};

use crate::colors::ColorPicker;

#[derive(Eq, Hash, PartialEq)]
pub enum LayerTypes {
    Commits = 0,
    Merges = 1,
    Pipes = 2,
}

pub struct LayerBuilder<'a> {
    layers: HashMap<LayerTypes, Vec<(String, Color)>>,
    color: &'a RefCell<ColorPicker>,
}

impl<'a> LayerBuilder<'a> {
    pub fn new(color: &'a RefCell<ColorPicker>) -> Self {
        Self {
            layers: HashMap::new(),
            color,
        }
    }

    pub fn add(
        &mut self,
        layer: LayerTypes,
        symbol: String,
        lane_idx: usize,
        custom: Option<Color>,
    ) {
        self.layers
            .entry(layer)
            .or_default()
            .push((symbol, custom.unwrap_or(self.color.borrow().get(lane_idx))));
    }
}

// Context struct holding mutable reference to LayerBuilder
pub struct LayersCtx<'a> {
    pub builder: LayerBuilder<'a>,
}

impl<'a> LayersCtx<'a> {
    pub fn clear(&mut self) {
        self.builder.layers.clear();
    }
    pub fn commit(&mut self, sym: &str, lane: usize) {
        self.builder
            .add(LayerTypes::Commits, sym.to_string(), lane, None);
    }
    pub fn pipe(&mut self, sym: &str, lane: usize) {
        self.builder
            .add(LayerTypes::Pipes, sym.to_string(), lane, None);
    }
    pub fn merge(&mut self, sym: &str, lane: usize) {
        self.builder
            .add(LayerTypes::Merges, sym.to_string(), lane, None);
    }
    pub fn pipe_custom(&mut self, sym: &str, lane: usize, color: Color) {
        self.builder
            .add(LayerTypes::Pipes, sym.to_string(), lane, Some(color));
    }
    pub fn bake(&mut self, spans: &mut Vec<Span>) {
        // Determine max length across all layers
        let max_len = [LayerTypes::Commits, LayerTypes::Merges, LayerTypes::Pipes]
            .iter()
            .filter_map(|layer| self.builder.layers.get(layer))
            .map(|tokens| tokens.len())
            .max()
            .unwrap_or(0);

        // For each token
        for token_index in 0..max_len {
            let mut symbol = " ";
            let mut color: Color = Color::Black;

            // For each layer
            for layer in [LayerTypes::Commits, LayerTypes::Merges, LayerTypes::Pipes] {
                if let Some(tokens) = self.builder.layers.get(&layer)
                    && token_index < tokens.len()
                {
                    // If the layer has a token at this index
                    if let Some((_symbol, _color)) = tokens.get(token_index)
                        && _symbol.trim() != ""
                    {
                        symbol = _symbol;
                        color = *_color;
                        break;
                    }
                }
            }
            spans.push(Span::styled(symbol.to_string(), Style::default().fg(color)));
        }
    }
}

// Macro to create a context and execute a block with it
#[macro_export]
macro_rules! layers {
    ($color:expr) => {{
        let builder = $crate::graph::layers::LayerBuilder::new($color);
        let ctx = $crate::graph::layers::LayersCtx { builder };
        ctx
    }};
}
