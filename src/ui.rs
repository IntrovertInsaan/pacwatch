use crate::app::{App, Focus};
use crate::pacman::Package;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[0]);

    // 1. Categories
    let cat_title = if app.focus == Focus::Categories { "Categories (Focused)" } else { "Categories" };
    let cat_list: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(c.as_str())).collect();
    f.render_stateful_widget(
        List::new(cat_list)
        .block(Block::default().borders(Borders::ALL).title(cat_title))
        .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue)),
        chunks[0],
        &mut app.cat_state,
    );

    // 2. Packages
    let pkg_title = if app.focus == Focus::Packages { "Packages (Focused)" } else { "Packages" };
    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| ListItem::new(app.all_packages[i].name.as_str()))
        .collect();
    f.render_stateful_widget(
        List::new(pkg_list)
        .block(Block::default().borders(Borders::ALL).title(pkg_title))
        .highlight_symbol(">> "),
        chunks[1],
        &mut app.pkg_state,
    );

    // 3. Details
    draw_detail(f, chunks[2], app.selected_package());

    // 4. Status Bar
    f.render_widget(Paragraph::new("Tab to switch | q to quit"), main_chunks[1]);
}

fn draw_detail(f: &mut Frame, area: Rect, pkg: Option<&Package>) {
    let block = Block::default().borders(Borders::ALL).title("Details");
    if let Some(p) = pkg {
        let text = format!("Name: {}\nVersion: {}\nDesc: {}", p.name, p.version, p.description);
        f.render_widget(Paragraph::new(text).block(block), area);
    } else {
        f.render_widget(block, area);
    }
}
