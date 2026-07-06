use crate::app::{App, Focus};
use crate::pacman::{format_epoch, human_size};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const HIGHLIGHT_BG: Color = Color::Cyan;
const HIGHLIGHT_FG: Color = Color::Black;
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
    let size = f.area();
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

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

    if app.show_help {
        draw_help_overlay(f, size);
    }
}

fn draw_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    if let Some(_) = &app.input_mode {
        let text = Span::styled(format!(" New category: {}", app.input_buffer), Style::default().fg(Color::Yellow));
        f.render_widget(Paragraph::new(text).block(block("Assign Category", true)), area);
        return;
    }

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
    let usable = area.width.saturating_sub(2 + 2) as usize;

    let items: Vec<ListItem> = app.categories.iter()
        .map(|c| {
            let size = human_size(app.category_size(c));
            let name_budget = usable.saturating_sub(size.len() + 1);
            let name = if c.chars().count() > name_budget {
                let t: String = c.chars().take(name_budget.saturating_sub(1)).collect();
                format!("{}…", t)
            } else {
                c.clone()
            };
            let gap = usable.saturating_sub(name.chars().count() + size.len()).max(1);
            ListItem::new(Line::from(vec![
                    Span::raw(name),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(size, Style::default().fg(DIM)),
            ]))
        })
    .collect();

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
    let usable = area.width.saturating_sub(2 + 2) as usize;

    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| {
            let p = &app.all_packages[i];
            let is_orphan = p.is_orphan();
            let name_style = if is_orphan {
                Style::default().fg(Color::Red)
            } else if p.install_reason != "Explicitly installed" {
                Style::default().fg(DEP_COLOR)
            } else {
                Style::default().fg(Color::White)
            };
            let cat_tag = if filtering { format!(" [{}]", app.category_map.get(&p.name)) } else { String::new() };
            let size = human_size(p.installed_size);
            let name_budget = usable.saturating_sub(size.len() + cat_tag.len() + 1);
            let name = if p.name.chars().count() > name_budget {
                format!("{}…", p.name.chars().take(name_budget.saturating_sub(1)).collect::<String>())
            } else {
                p.name.clone()
            };
            let gap = usable.saturating_sub(name.chars().count() + size.len() + cat_tag.len()).max(1);
            let mut spans = vec![
                Span::styled(name, name_style),
                Span::raw(" ".repeat(gap)),
                Span::styled(size, Style::default().fg(DIM)),
            ];
            if filtering {
                spans.push(Span::styled(cat_tag, Style::default().fg(ACCENT)));
            }
            let base_style = if app.marked.contains(&p.name) {
                Style::default().bg(Color::Rgb(60, 50, 10))
            } else {
                Style::default()
            };
            ListItem::new(Line::from(spans)).style(base_style)
        })
    .collect();

    let count_title = format!(
        "Packages ({}/{}) [sort: {}]",
        app.filtered.len(),
        app.all_packages.len(),
        app.sort_key.label()
    );

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
    let title = if focused { "Details (j/k to scroll)" } else { "Details" };
    let block_widget = block(title, focused);

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

    let visible_height = area.height.saturating_sub(2); // minus top/bottom border
    let max_scroll = (lines.len() as u16).saturating_sub(visible_height);
    let scroll = app.detail_scroll.min(max_scroll);

    let p = Paragraph::new(lines)
        .block(block_widget)
        .wrap(Wrap { trim: true })
        .scroll((scroll, 0));
    f.render_widget(p, area);

    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(if focused { ACCENT } else { DIM }));
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

fn draw_statusbar(f: &mut Frame, _app: &App, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" h/l", Style::default().fg(ACCENT)),
        Span::raw(" switch pane  "),
        Span::styled("/", Style::default().fg(ACCENT)),
        Span::raw(" filter  "),
        Span::styled("s", Style::default().fg(ACCENT)),
        Span::raw(" sort  "),
        Span::styled("r", Style::default().fg(ACCENT)),
        Span::raw(" reload categories.toml  "),
        Span::styled(".", Style::default().fg(ACCENT)),
        Span::raw(" toggle deps  "),
        Span::styled("?", Style::default().fg(ACCENT)),
        Span::raw(" help  "),
        Span::styled("q", Style::default().fg(ACCENT)),
        Span::raw(" quit"),
    ]);
    f.render_widget(Paragraph::new(text).style(Style::default().fg(DIM)), area);
}

fn draw_help_overlay(f: &mut Frame, size: Rect) {
    let width = 62.min(size.width.saturating_sub(4));
    let height = 16.min(size.height.saturating_sub(4));
    let area = Rect {
        x: (size.width.saturating_sub(width)) / 2,
        y: (size.height.saturating_sub(height)) / 2,
        width,
        height,
    };

    let header = |title: &str| Line::from(Span::styled(title.to_string(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)));
    let key = |k: &str, desc: &str| Line::from(format!("    {:<16}{}", k, desc));

    let lines = vec![
        header("Navigation"),
        key("h/l", "Switch focus between panes"),
        key("j/k", "Move selection, or scroll Details"),
        key("gg/G", "Jump to top / bottom of current pane"),
        Line::from(""),
        header("Search"),
        key("/", "Search package names"),
        key("d:<text>", "Search descriptions"),
        key("c:<text>", "Search categories"),
        key("Enter", "Finish search"),
        key("Esc", "Cancel search"),
        Line::from(""),
        header("Actions"),
        key(".", "Toggle dependency-tail packages"),
        key("o", "Toggle orphans-only (unneeded deps)"),
        key("s", "Cycle sort: name / size / installed / reason"),
        key("r", "Reload categories.toml"),
        key("?", "Toggle this help"),
        key("q", "Quit"),
    ];

    let p = Paragraph::new(lines)
        .block(block("Help", true))
        .wrap(Wrap { trim: false });
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}
