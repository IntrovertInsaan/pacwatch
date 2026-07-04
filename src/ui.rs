use crate::app::{App, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.area());

    let cat_title = if app.focus == Focus::Categories { "Categories (Focused)" } else { "Categories" };
    f.render_widget(Block::default().borders(Borders::ALL).title(cat_title), chunks[0]);

    let pkg_title = if app.focus == Focus::Packages { "Packages (Focused)" } else { "Packages" };
    let content = format!("Selected package index: {}", app.package_state);
    f.render_widget(
        Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(pkg_title)), 
        chunks[1]
    );
}
