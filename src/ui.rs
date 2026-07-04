use crate::app::{App, Focus};
use crate::pacman::Package;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Layout: The 3 columns for Categories, Packages, Details
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ])
        .split(main_layout[1]);

    // 1. Render Filter Bar
    let filter_title = if app.focus == Focus::Filter { "Filter (Focused)" } else { "Filter" };
    let filter_content = if app.filter_text.is_empty() { "Type / to search..." } else { &app.filter_text };
    f.render_widget(
        Paragraph::new(filter_content)
            .block(Block::default().borders(Borders::ALL).title(filter_title)),
        main_layout[0],
    );

    // 2. Render Categories
    let cat_title = if app.focus == Focus::Categories { "Categories (Focused)" } else { "Categories" };
    let cat_list: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(c.as_str())).collect();
    f.render_stateful_widget(
        List::new(cat_list)
            .block(Block::default().borders(Borders::ALL).title(cat_title))
            .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue)),
        columns[0],
        &mut app.cat_state,
    );

    // 3. Render Packages
    let pkg_title = if app.focus == Focus::Packages { "Packages (Focused)" } else { "Packages" };
    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| ListItem::new(app.all_packages[i].name.as_str()))
        .collect();
    f.render_stateful_widget(
        List::new(pkg_list)
            .block(Block::default().borders(Borders::ALL).title(pkg_title))
            .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue)),
        columns[1],
        &mut app.pkg_state,
    );

    // 4. Render Details
    draw_detail(f, columns[2], app.selected_package());

    // 5. Render Status Bar
    f.render_widget(Paragraph::new("h/l: Switch pane | j/k: Scroll | /: Filter | q: Quit"), main_layout[2]);
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
