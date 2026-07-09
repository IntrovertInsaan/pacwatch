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
const MARKED_BG: Color = Color::Rgb(60, 50, 10);
const ERROR_COLOR: Color = Color::Red;
const SECTION_COLOR: Color = Color::White;
const INPUT_ACCENT: Color = Color::Rgb(140, 170, 230);

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
        draw_help_overlay(f, app, size);
    }
}

fn draw_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    if let Some(mode) = app.input_mode {
        let (title, content) = match mode {
            crate::app::InputMode::AddCategory => (
                "Create Category",
                format!(" Name: {}", app.input_buffer),
            ),

            crate::app::InputMode::RenameCategory => (
                "Rename Category",
                format!(" Name: {}", app.input_buffer),
            ),

            crate::app::InputMode::DeleteCategory => (
                "Delete Category",
                format!(" Delete \"{}\"? [y/n]", app.categories[app.selected_category]),
            ),
        };

        let style = match mode {
            crate::app::InputMode::DeleteCategory => Style::default().fg(Color::White),
            _ => Style::default().fg(INPUT_ACCENT),
        };

        let text = Span::styled(content, style);

        f.render_widget(
            Paragraph::new(text).block(block(title, true)),
            area,
        );
        return;
    }

    let focused = app.focus == Focus::Filter;

    let (title, content) = if focused {
        (
            "Search",
            if app.filter_text.is_empty() {
                " Type to search…".to_string()
            } else {
                format!(" {}_", app.filter_text)
            },
        )
    } else {
        (
            "pacwatch",
            " Type / to search (c: category, d: description)".to_string(),
        )
    };

    let text = if focused {
        if app.filter_text.is_empty() {
            Span::styled(content, Style::default().fg(DIM))
        } else {
            Span::styled(content, Style::default().fg(INPUT_ACCENT))
        }
    } else {
        Span::styled(content, Style::default().fg(DIM))
    };

    f.render_widget(
        Paragraph::new(text).block(block(title, focused)),
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

fn build_packages_title(app: &App, area_width: u16) -> String {
    let usable = area_width.saturating_sub(2) as usize;
    let base = format!("Packages ({}/{})", app.filtered.len(), app.all_packages.len());

    let extra = if !app.marked.is_empty() {
        format!(" [{} marked]", app.marked.len())
    } else {
        format!(" [sort: {}]", app.sort_key.label())
    };

    let mut title = base;
    if title.chars().count() + extra.chars().count() <= usable {
        title.push_str(&extra);
    }
    title
}

fn draw_packages(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::Packages;
    let filtering = !app.filter_text.is_empty();
    let has_marks = !app.marked.is_empty();
    let usable = area.width.saturating_sub(2 + 2 + if has_marks { 2 } else { 0 }) as usize;

    let pkg_list: Vec<ListItem> = app.filtered.iter()
        .map(|&i| {
            let p = &app.all_packages[i];
            let is_orphan = p.is_orphan();
            let name_style = if is_orphan {
                Style::default().fg(Color::Red)
            } else if !p.install_reason.is_explicit() {
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
            let mut spans = if has_marks {
                let mark_glyph = if app.marked.contains(&p.name) { "✓ " } else { "  " };
                vec![Span::styled(mark_glyph, Style::default().fg(ACCENT))]
            } else {
                vec![]
            };
            spans.push(Span::styled(name, name_style));
            spans.push(Span::raw(" ".repeat(gap)));
            spans.push(Span::styled(size, Style::default().fg(DIM)));
            if filtering {
                spans.push(Span::styled(cat_tag, Style::default().fg(ACCENT)));
            }
            let base_style = if app.marked.contains(&p.name) {
                Style::default().bg(MARKED_BG)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(spans)).style(base_style)
        })
    .collect();

    let count_title = build_packages_title(app, area.width);

    if app.filtered.is_empty() {
        let hint = if !app.filter_text.is_empty() {
            "No packages match this search.\nPress Esc to clear it.".to_string()
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

    let none_or = |v: &[String]| {
        if v.is_empty() {
            "None".to_string()
        } else {
            format!("({}) {}", v.len(), v.join(", "))
        }
    };
    let divider_width = area.width.saturating_sub(2) as usize;
    let content_width = (area.width as usize).saturating_sub(2).saturating_sub(16).max(10);

    let field = |label: &str, value: String| -> Vec<Line<'static>> {
        let mut chunks = Vec::new();
        let mut current = String::new();
        for word in value.split_whitespace() {
            if word.chars().count() > content_width {
                if !current.is_empty() {
                    chunks.push(current.clone());
                    current.clear();
                }
                let mut remaining: &str = word;
                while remaining.chars().count() > content_width {
                    let (head, tail) = remaining.split_at(
                        remaining.char_indices().nth(content_width).map(|(i, _)| i).unwrap_or(remaining.len())
                    );
                    chunks.push(head.to_string());
                    remaining = tail;
                }
                current = remaining.to_string();
                continue;
            }
            let candidate = if current.is_empty() { word.to_string() } else { format!("{} {}", current, word) };
            if candidate.chars().count() > content_width {
                chunks.push(current.clone());
                current = word.to_string();
            } else {
                current = candidate;
            }
        }
        chunks.push(current);
        chunks.into_iter().enumerate().map(|(i, text)| {
            if i == 0 {
                Line::from(vec![
                    Span::styled(format!("{:<14}", label), Style::default().fg(ACCENT)),
                    Span::styled(": ", Style::default().fg(DIM)),
                    Span::raw(text),
                ])
            } else {
                Line::from(vec![Span::raw(" ".repeat(16)), Span::raw(text)])
            }
        }).collect()
    };
    let section = |title: &str| vec![Line::from(Span::styled(
            title.to_string(),
            Style::default().fg(SECTION_COLOR).add_modifier(Modifier::BOLD),
    ))];

    let mut lines: Vec<Line> = vec![
        vec![Line::from(vec![
            Span::styled(format!("󰆧 {} ", pkg.name), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(pkg.version.clone(), Style::default().fg(DIM)),
        ])],
        vec![Line::from(Span::styled(pkg.description.clone(), Style::default().fg(DIM)))],
        vec![Line::from(Span::styled("─".repeat(divider_width), Style::default().fg(DIM)))],
        vec![Line::from("")],

        section("[Package]"),
        field("Category", app.category_map.get(&pkg.name).to_string()),
        field("Architecture", pkg.architecture.clone()),
        field("URL", pkg.url.clone()),
        field("Licenses", none_or(&pkg.licenses)),
        field("Groups", none_or(&pkg.groups)),
        field("Provides", none_or(&pkg.provides)),
        field("Install Size", human_size(pkg.installed_size)),
        field("Packager", pkg.packager.clone()),
        field("Build Date", format_epoch(pkg.build_date)),
        field("Install Date", format_epoch(pkg.install_date)),
        field("Install Reason", pkg.install_reason.label().to_string()),
        field("Install Script", if pkg.has_install_script { "Yes".to_string() } else { "No".to_string() }),
        field("Validated By", if pkg.validated_by.is_empty() { "None".to_string() } else { pkg.validated_by.clone() }),
        vec![Line::from("")],

        section("[Dependencies]"),
        field("Depends On", none_or(&pkg.depends)),
        field("Optional Deps", none_or(&pkg.optdepends)),
        field("Required By", none_or(&pkg.required_by)),
        field("Optional For", none_or(&pkg.optional_for)),
        field("Conflicts With", none_or(&pkg.conflicts)),
        field("Replaces", none_or(&pkg.replaces)),
        vec![Line::from("")],

        section(&format!("[Files]({})", pkg.files.len())),
        ].into_iter().flatten().collect();
        lines.extend(pkg.files.iter().map(|f| Line::from(f.as_str())));

        let visible_height = area.height.saturating_sub(2);
        let max_scroll = (lines.len() as u16).saturating_sub(visible_height);
        let scroll = app.detail_scroll.min(max_scroll);

    let p = Paragraph::new(lines)
        .block(block_widget)
        .wrap(Wrap { trim: false })
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

fn draw_statusbar(f: &mut Frame, app: &App, area: Rect) {
    const HINTS: &[(&str, &str)] = &[
        ("hjkl", "navigate"),
        ("/", "search"),
        ("s", "sort"),
        (".", "deps"),
        ("o", "orphans"),
        ("r", "reload"),
        ("?", "help"),
        ("q", "quit"),
    ];

    let status_text = app.status.as_ref().map(|s| format!("{} ", s.text));
    let status_len = status_text.as_ref().map_or(0, |s| s.chars().count());
    let width = area.width as usize;

    let budget = width.saturating_sub(status_len);
    let mut hint_spans = Vec::new();
    let mut used = 0usize;

    for (key, label) in HINTS {
        let piece_len = key.chars().count() + label.chars().count() + 2;
        if used + piece_len > budget {
            break;
        }
        hint_spans.push(Span::raw(" "));
        hint_spans.push(Span::styled(*key, Style::default().fg(ACCENT)));
        hint_spans.push(Span::raw(" "));
        hint_spans.push(Span::raw(*label));
        used += piece_len;
    }

    let mut spans = hint_spans;
    if let Some(status_text) = status_text {
        let color = match app.status.as_ref().unwrap().level {
            crate::app::StatusLevel::Info => ACCENT,
            crate::app::StatusLevel::Error => ERROR_COLOR,
        };
        let gap = width.saturating_sub(used + status_len).max(1);
        spans.push(Span::raw(" ".repeat(gap)));
        spans.push(Span::styled(status_text, Style::default().fg(color)));
    }

    let text = Line::from(spans);
    f.render_widget(Paragraph::new(text).style(Style::default().fg(DIM)), area);
}

fn draw_help_overlay(f: &mut Frame, app: &App, size: Rect) {
    let dim = Block::default().style(Style::default().fg(DIM).bg(Color::Reset));
    f.render_widget(dim, size);

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
        key("h / l", "Switch pane"),
        key("j / k", "Move selection"),
        key("gg / G", "Jump top / bottom"),
        Line::from(""),

        header("Search"),
        key("/", "Search packages"),
        key("d:<text>", "Search descriptions"),
        key("c:<text>", "Search categories"),
        key("Enter", "Finish search"),
        key("Esc", "Cancel search"),
        Line::from(""),

        header("Categories"),
        key("a", "Create category"),
        key("r", "Rename category"),
        key("d", "Delete category"),
        key("Space", "Mark package"),
        key("Enter", "Move marked packages"),
        key("M", "Clear all marks"),
        Line::from(""),

        header("Packages"),
        key("s", "Cycle sort: name/size/new"),
        key(".", "Toggle dependencies"),
        key("o", "Toggle orphans"),
        key("R", "Reload categories.toml"),
        Line::from(""),

        header("General"),
        key("?", "Help"),
        key("q", "Quit"),
    ];

    let visible_height = area.height.saturating_sub(2);
    let max_scroll = (lines.len() as u16).saturating_sub(visible_height);
    let scroll = app.help_scroll.min(max_scroll);

    let p = Paragraph::new(lines)
        .block(block("Help", true))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(Clear, area);
    f.render_widget(p, area);

    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::new(max_scroll as usize).position(scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .style(Style::default().fg(ACCENT));
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}
