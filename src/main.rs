#[rustfmt::skip]
use std::io;
#[rustfmt::skip]
mod app {
    pub mod app;
    pub mod app_default;
    pub mod app_input;
    pub mod app_draw;
    pub mod layout {
        pub mod layout;
        pub mod title;
        pub mod status;
    }
}
mod core {
    pub mod buffer;
    pub mod chunk;
    pub mod layers;
    pub mod renderers;
    pub mod walker;
}
pub mod git {
    pub mod actions;
    pub mod queries;
}
pub mod utils {
    pub mod colors;
    pub mod symbols;
    pub mod time;
}

use crate::app::app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
