mod app;
mod categories;
mod pacman;
mod ui;

use clap::Parser;
use crossterm::{event::{self, Event, KeyEventKind}, execute, terminal::*,};
use ratatui::prelude::*;
use std::{io::{self, stdout}, time::Duration};

#[derive(Parser)]
#[command(name = "pacwatch", about = "A categorized TUI browser for pacman packages")]
struct Cli {
    #[arg(long)]
    config_path: bool,
    #[arg(long)]
    reset_config: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    if cli.config_path {
        println!("{}", categories::config_path().display());
        return Ok(());
    }
    if cli.reset_config {
        categories::reset_config()?;
        println!("Reset {} to bundled defaults.", categories::config_path().display());
        return Ok(());
    }

    let map = categories::load();
    let pkgs = pacman::load_installed_packages()?;
    let mut app = app::App::new(pkgs, map);

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                app.handle_key(key);
            }
        }
        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
