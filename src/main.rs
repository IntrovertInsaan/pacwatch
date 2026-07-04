mod app;
mod categories;
mod pacman;
mod ui;

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
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Tab => app.focus = if app.focus == app::Focus::Categories { app::Focus::Packages } else { app::Focus::Categories },

                    KeyCode::Down | KeyCode::Char('j') => app.move_package(1),
                    KeyCode::Up | KeyCode::Char('k') => app.move_package(-1),
                    KeyCode::Left | KeyCode::Char('h') => app.move_category(-1),
                    KeyCode::Right | KeyCode::Char('l') => app.move_category(1),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
