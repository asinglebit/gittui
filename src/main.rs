mod app;
mod helpers;
mod core {
    pub mod buffer;
    pub mod chunk;
    pub mod layers;
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

use std::io;

use app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
