mod app;
mod exam;
mod score;
mod store;
mod text;
mod ui;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let exams = match exam::load_all() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to parse exam files: {e}");
            std::process::exit(1);
        }
    };
    let mut app = app::App::new(exams);
    let mut terminal = ratatui::init();
    let result = (|| -> std::io::Result<()> {
        while !app.quit {
            terminal.draw(|f| ui::draw(f, &app))?;
            let wait = if app.animating() { Duration::from_millis(40) } else { Duration::from_millis(250) };
            if event::poll(wait)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        app.on_key(key);
                    }
                }
            }
            app.tick();
        }
        Ok(())
    })();
    ratatui::restore();
    result
}
