#[rustfmt::skip]
use std::io;
#[rustfmt::skip]
mod app {
    pub mod app;
    pub mod app_default;
    pub mod app_input;
    pub mod app_layout;
    pub mod app_draw_title;
    pub mod app_draw_graph;
    pub mod app_draw_editor;
    pub mod app_draw_viewer;
    pub mod app_draw_inspector;
    pub mod app_draw_status;
    pub mod app_draw_statusbar;
    pub mod app_draw_modal_actions;
    pub mod app_draw_modal_checkout;
    pub mod app_draw_modal_commit;
}
mod core {
    pub mod buffer;
    pub mod chunk;
    pub mod layers;
    pub mod renderers;
    pub mod walker;
}
pub mod git {
    pub mod actions {
        pub mod commits;
    }
    pub mod queries {
        pub mod commits;
        pub mod diffs;
    }
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
