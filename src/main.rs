mod app;
mod categories;
mod pacman;
mod ui;

use app::Focus;
use crossterm::{event::{self, Event, KeyCode}, execute, terminal::*,};
use ratatui::prelude::*;
use std::{io::{self, stdout}, time::Duration};

fn main() -> io::Result<()> {
    let map = categories::load();
    let pkgs = pacman::load_installed_packages();
    let mut app = app::App::new(pkgs, map);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') && app.focus != Focus::Filter {
                    break;
                }
                app.handle_key(key);
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
