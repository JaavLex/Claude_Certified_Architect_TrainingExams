mod app;
mod exam;
mod paths;
mod score;
mod store;
mod text;
mod train;
mod ui;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use std::time::Duration;

const HELP: &str = "\
exam-tui — timed CCAR-F practice exams in the terminal

USAGE:
    exam-tui [OPTIONS] [EXAM.json | FOLDER]...

Loads the two built-in practice exams, every *.json exam in
    {DIR}
and any exam files or folders given as arguments.

OPTIONS:
    --check        Validate the exam files and exit
    -V, --version  Show the version
    -h, --help     Show this help
";

fn main() -> std::io::Result<()> {
    let mut paths = Vec::new();
    let mut check = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", HELP.replace("{DIR}", &exam::user_exam_dir().display().to_string()));
                return Ok(());
            }
            "-V" | "--version" => {
                println!("exam-tui {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--check" => check = true,
            _ => paths.push(std::path::PathBuf::from(arg)),
        }
    }
    if check {
        // Validate exam files without starting the UI.
        let loaded = exam::load_all(&paths);
        for e in &loaded.exams {
            println!(
                "ok     {} — {} ({} items, {} min) [{}]",
                e.id,
                e.title,
                e.questions.len(),
                e.time_limit_minutes,
                e.source
            );
        }
        for err in &loaded.errors {
            println!("error  {err}");
        }
        std::process::exit(if loaded.errors.is_empty() { 0 } else { 1 });
    }
    let mut app = app::App::new(paths);
    let mut terminal = ratatui::init();
    // Paste a path in one piece. Not every terminal supports this (the legacy
    // Windows console does not); pasted text then arrives as key presses.
    let _ = ratatui::crossterm::execute!(std::io::stdout(), ratatui::crossterm::event::EnableBracketedPaste);
    let result = (|| -> std::io::Result<()> {
        while !app.quit {
            terminal.draw(|f| ui::draw(f, &app))?;
            let wait = if app.animating() { Duration::from_millis(40) } else { Duration::from_millis(250) };
            if event::poll(wait)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => app.on_key(key),
                    Event::Paste(text) => app.on_paste(&text),
                    _ => {}
                }
            }
            app.tick();
        }
        Ok(())
    })();
    let _ = ratatui::crossterm::execute!(std::io::stdout(), ratatui::crossterm::event::DisableBracketedPaste);
    ratatui::restore();
    result
}
