use crate::app::{App, Focus};
use crate::pacman::Package;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const HIGHLIGHT_BG: Color = Color::Cyan;
const HIGHLIGHT_FG: Color = Color::Black;

fn block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(DIM)
    };
    Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(border_style)
}

pub fn draw(f: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_filter_bar(f, app, main_layout[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(50),
        ])
        .split(main_layout[1]);

    draw_categories(f, app, columns[0]);
    draw_packages(f, app, columns[1]);
    draw_detail(f, app, columns[2]);
    draw_statusbar(f, app, main_layout[2]);
}

fn draw_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Filter;
    let text = if app.filter_text.is_empty() && !focused {
        Span::styled(" Type / to search...", Style::default().fg(DIM))
    } else {
        Span::styled(format!(" {}", app.filter_text), Style::default().fg(Color::White))
    };
    f.render_widget(
        Paragraph::new(text).block(block("pacwatch", focused)),
        area,
    );
}

fn draw_categories(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Categories;
    let items: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(c.as_str())).collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_category));

    let list = List::new(items)
        .block(block("Categories", focused))
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(HIGHLIGHT_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_packages(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Packages;
    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| ListItem::new(app.all_packages[i].name.as_str()))
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.package_state));

    let list = List::new(pkg_list)
        .block(block("Packages", focused))
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(HIGHLIGHT_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Detail;
    draw_detail_content(f, area, app.selected_package(), block("Details", focused));
}

fn draw_detail_content(f: &mut Frame, area: Rect, pkg: Option<&Package>, block: Block) {
    if let Some(p) = pkg {
        let text = format!("Name: {}\nVersion: {}\nDesc: {}", p.name, p.version, p.description);
        f.render_widget(Paragraph::new(text).block(block), area);
    } else {
        f.render_widget(block, area);
    }
}

fn draw_statusbar(f: &mut Frame, _app: &App, area: Rect) {
    f.render_widget(
        Paragraph::new("h/l: Switch pane | j/k: Scroll | /: Filter | q: Quit")
            .style(Style::default().fg(DIM)),
        area,
    );
}
