use crate::app::{App, Focus};
use crate::pacman::{format_epoch, human_size};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const HIGHLIGHT_BG: Color = Color::Cyan;
const HIGHLIGHT_FG: Color = Color::Black;
/// Desaturated teal for dependency-tail packages when shown via '.' --
/// distinct from explicit packages' plain white without just being gray.
const DEP_COLOR: Color = Color::Rgb(94, 138, 138);

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
    let filtering = !app.filter_text.is_empty();
    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| {
            let p = &app.all_packages[i];
            let is_dep = p.install_reason != "Explicitly installed";
            let name_style = if is_dep {
                Style::default().fg(DEP_COLOR)
            } else {
                Style::default().fg(Color::White)
            };
            let mut spans = vec![
                Span::styled(p.name.clone(), name_style),
                Span::styled(format!("  {}", p.version), Style::default().fg(DIM)),
            ];
            // Results can come from any category while filtering, so tag each
            // one -- otherwise there's no way to tell where a match lives.
            if filtering {
                let cat = app.category_map.get(&p.name);
                spans.push(Span::styled(format!("  [{}]", cat), Style::default().fg(ACCENT)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let count_title = if app.show_dependencies {
        format!("Packages ({}/{}) [ . for hide deps]", app.filtered.len(), app.all_packages.len())
    } else {
        format!("Packages ({}/{}) [ . for show deps]", app.filtered.len(), app.all_packages.len())
    };

    if app.filtered.is_empty() {
        let hint = if !app.filter_text.is_empty() {
            "No packages match this filter.".to_string()
        } else if !app.show_dependencies {
            "Nothing explicitly installed here.\nPress '.' to also show dependency packages.".to_string()
        } else {
            "No packages in this category.".to_string()
        };
        let p = Paragraph::new(hint)
            .style(Style::default().fg(DIM))
            .block(block(&count_title, focused));
        f.render_widget(p, area);
        return;
    }

    let mut state = ListState::default();
    state.select(Some(app.package_state));

    let list = List::new(pkg_list)
        .block(block(&count_title, focused))
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(HIGHLIGHT_FG).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Detail;
    let block_widget = block("Details", focused);

    let Some(pkg) = app.selected_package() else {
        let p = Paragraph::new("No package selected").block(block_widget);
        f.render_widget(p, area);
        return;
    };

    let field = |label: &str, value: String| {
        Line::from(vec![
            Span::styled(format!("{:<14}", label), Style::default().fg(ACCENT)),
            Span::styled(": ", Style::default().fg(DIM)),
            Span::raw(value),
        ])
    };
    let none_or = |v: &[String]| if v.is_empty() { "None".to_string() } else { v.join(", ") };

    let mut lines = vec![
        field("Name", pkg.name.clone()),
        field("Version", pkg.version.clone()),
        field("Category", app.category_map.get(&pkg.name).to_string()),
        field("Description", pkg.description.clone()),
        field("Architecture", pkg.architecture.clone()),
        field("URL", pkg.url.clone()),
        field("Licenses", none_or(&pkg.licenses)),
        field("Groups", none_or(&pkg.groups)),
        field("Provides", none_or(&pkg.provides)),
        field("Depends On", none_or(&pkg.depends)),
        field("Optional Deps", none_or(&pkg.optdepends)),
        field("Required By", none_or(&pkg.required_by)),
        field("Optional For", none_or(&pkg.optional_for)),
        field("Conflicts With", none_or(&pkg.conflicts)),
        field("Replaces", none_or(&pkg.replaces)),
        field("Installed Size", human_size(pkg.installed_size)),
        field("Packager", pkg.packager.clone()),
        field("Build Date", format_epoch(pkg.build_date)),
        field("Install Date", format_epoch(pkg.install_date)),
        field("Install Reason", pkg.install_reason.clone()),
        field("Validated By", if pkg.validated_by.is_empty() { "None".to_string() } else { pkg.validated_by.clone() }),
        Line::from(""),
        Line::from(Span::styled(format!("Files ({}):", pkg.files.len()), Style::default().fg(ACCENT))),
    ];
    lines.extend(pkg.files.iter().map(|f| Line::from(f.as_str())));

    let p = Paragraph::new(lines).block(block_widget);
    f.render_widget(p, area);
}

fn draw_statusbar(f: &mut Frame, _app: &App, area: Rect) {
    f.render_widget(
        Paragraph::new("h/l: Switch pane | j/k: Scroll | /: Filter | q: Quit")
            .style(Style::default().fg(DIM)),
        area,
    );
}
