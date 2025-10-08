mod app;
mod colors;
mod helpers;
mod graph {
    pub mod chunk;
}

use std::io;

use app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
