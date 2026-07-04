use crate::app::{App, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.area());

    let cat_list: Vec<ListItem> = app.categories.iter()
        .map(|c| ListItem::new(c.as_str()))
        .collect();
    let cat_title = if app.focus == Focus::Categories { "Categories (Focused)" } else { "Categories" };
    f.render_widget(List::new(cat_list).block(Block::default().borders(Borders::ALL).title(cat_title)), chunks[0]);

    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| ListItem::new(app.all_packages[i].name.as_str()))
        .collect();
    let pkg_title = if app.focus == Focus::Packages { "Packages (Focused)" } else { "Packages" };
    f.render_widget(List::new(pkg_list).block(Block::default().borders(Borders::ALL).title(pkg_title)), chunks[1]);
}
