use crate::categories::CategoryMap;
use crate::pacman::Package;
use crossterm::event::KeyCode;
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq)]
pub enum Focus {
    Categories,
    Packages,
    Detail,
    Filter,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SortKey {
    NameAsc,
    NameDesc,
    Size,
    Newest,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    AssignCategory,
    RenameCategory,
}

impl SortKey {
    fn next(self) -> Self {
        match self {
            SortKey::NameAsc => SortKey::NameDesc,
            SortKey::NameDesc => SortKey::Size,
            SortKey::Size => SortKey::Newest,
            SortKey::Newest => SortKey::NameAsc,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortKey::NameAsc => "a-z",
            SortKey::NameDesc => "z-a",
            SortKey::Size => "big",
            SortKey::Newest => "new",
        }
    }
}

pub struct App {
    pub all_packages: Vec<Package>,
    pub category_map: CategoryMap,
    pub categories: Vec<String>,
    pub selected_category: usize,
    pub filtered: Vec<usize>,
    pub filter_text: String,
    pub package_state: usize,
    pub detail_scroll: u16,
    pub focus: Focus,
    pub should_quit: bool,
    pub show_help: bool,
    pub pending_g: Option<Instant>,
    pub show_dependencies: bool,
    pub show_orphans_only: bool,
    pub sort_key: SortKey,
    pub input_mode: Option<InputMode>,
    pub input_buffer: String,
}

impl App {
    pub fn new(all_packages: Vec<Package>, category_map: CategoryMap) -> Self {
        let mut categories = vec!["All".to_string()];
        categories.extend(category_map.categories());
        let mut app = App {
            all_packages,
            category_map,
            categories,
            selected_category: 0,
            filtered: Vec::new(),
            filter_text: String::new(),
            package_state: 0,
            detail_scroll: 0,
            focus: Focus::Categories,
            should_quit: false,
            show_help: false,
            pending_g: None,
            show_dependencies: false,
            show_orphans_only: false,
            sort_key: SortKey::NameAsc,
            input_mode: None,
            input_buffer: String::new(),
        };
        app.recompute_filter();
        app
    }

    pub fn recompute_filter(&mut self) {
        let cat = self.categories[self.selected_category].clone();
        let query = self.filter_text.trim().to_lowercase();

        let (search_mode, needle) = if let Some(rest) = query.strip_prefix("d:") {
            ("description", rest.trim().to_string())
        } else if let Some(rest) = query.strip_prefix("c:") {
            ("category", rest.trim().to_string())
        } else {
            ("package", query.clone())
        };

        self.filtered = self.all_packages.iter().enumerate()
            .filter(|(_, p)| {
                let cat_ok = !needle.is_empty() || cat == "All" || self.category_map.get(&p.name) == cat;
                let text_ok = if needle.is_empty() {
                    true
                } else {
                    match search_mode {
                        "description" => p.description.to_lowercase().contains(&needle),
                        "category" => self.category_map.get(&p.name).to_lowercase().contains(&needle),
                        _ => p.name.to_lowercase().contains(&needle),
                    }
                };
                let reason_ok = self.show_dependencies || p.install_reason == "Explicitly installed";
                let orphan_ok = !self.show_orphans_only || p.is_orphan();
                cat_ok && text_ok && reason_ok && orphan_ok
            })
        .map(|(i, _)| i)
            .collect();

        let pkgs = &self.all_packages;
        match self.sort_key {
            SortKey::NameAsc => self.filtered.sort_by(|&a, &b| pkgs[a].name.cmp(&pkgs[b].name)),
            SortKey::NameDesc => self.filtered.sort_by(|&a, &b| pkgs[b].name.cmp(&pkgs[a].name)),
            SortKey::Size => self.filtered.sort_by(|&a, &b| pkgs[b].installed_size.cmp(&pkgs[a].installed_size)),
            SortKey::Newest => self.filtered.sort_by(|&a, &b| pkgs[b].install_date.cmp(&pkgs[a].install_date)),
        }

        if self.package_state >= self.filtered.len() {
            self.package_state = self.filtered.len().saturating_sub(1);
        }
        self.sync_category_cursor();
    }

    fn sync_category_cursor(&mut self) {
        if self.filter_text.is_empty() {
            return;
        }
        let Some(pkg) = self.selected_package() else {
            return;
        };
        let cat = self.category_map.get(&pkg.name).to_string();
        if let Some(idx) = self.categories.iter().position(|c| *c == cat) {
            self.selected_category = idx;
        }
    }

    pub fn move_package(&mut self, delta: i32) {
        if self.filtered.is_empty() { return; }
        let next = (self.package_state as i32 + delta).clamp(0, self.filtered.len() as i32 - 1);
        self.package_state = next as usize;
        self.detail_scroll = 0;
        self.sync_category_cursor();
    }

    pub fn scroll_detail(&mut self, delta: i32) {
        let next = self.detail_scroll as i32 + delta;
        self.detail_scroll = next.max(0) as u16;
    }

    pub fn jump_top(&mut self) {
        match self.focus {
            Focus::Categories => {
                self.filter_text.clear();
                self.selected_category = 0;
                self.package_state = 0;
                self.detail_scroll = 0;
                self.recompute_filter();
            }
            Focus::Packages => {
                self.package_state = 0;
                self.detail_scroll = 0;
                self.sync_category_cursor();
            }
            Focus::Detail => self.detail_scroll = 0,
            Focus::Filter => {}
        }
    }

    pub fn jump_bottom(&mut self) {
        match self.focus {
            Focus::Categories => {
                self.filter_text.clear();
                self.selected_category = self.categories.len().saturating_sub(1);
                self.package_state = 0;
                self.detail_scroll = 0;
                self.recompute_filter();
            }
            Focus::Packages => {
                self.package_state = self.filtered.len().saturating_sub(1);
                self.detail_scroll = 0;
                self.sync_category_cursor();
            }
            Focus::Detail => self.detail_scroll = u16::MAX,
            Focus::Filter => {}
        }
    }

    pub fn move_category(&mut self, delta: i32) {
        let len = self.categories.len() as i32;
        if len == 0 { return; }
        self.filter_text.clear();
        let mut next = self.selected_category as i32 + delta;
        next = next.clamp(0, len - 1);
        self.selected_category = next as usize;
        self.package_state = 0;
        self.detail_scroll = 0;
        self.recompute_filter();
    }

    pub fn category_size(&self, category: &str) -> u64 {
        self.all_packages.iter()
            .filter(|p| category == "All" || self.category_map.get(&p.name) == category)
            .filter(|p| self.show_dependencies || p.install_reason == "Explicitly installed")
            .map(|p| p.installed_size)
            .sum()
    }

    pub fn toggle_dependencies(&mut self) {
        self.show_dependencies = !self.show_dependencies;
        self.recompute_filter();
    }

    pub fn toggle_orphans_only(&mut self) {
        self.show_orphans_only = !self.show_orphans_only;
        self.recompute_filter();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_key = self.sort_key.next();
        self.recompute_filter();
    }

    pub fn start_assign_category(&mut self) {
        if self.selected_package().is_none() { return; }
        self.input_mode = Some(InputMode::AssignCategory);
        self.input_buffer.clear();
    }

    pub fn start_rename_category(&mut self) {
        let current = self.categories[self.selected_category].clone();
        if current == "All" || current == "Uncategorized" { return; }
        self.input_mode = Some(InputMode::RenameCategory);
        self.input_buffer = current.clone();
    }

    pub fn delete_selected_category(&mut self) {
        let current = self.categories[self.selected_category].clone();
        if current == "All" || current == "Uncategorized" { return; }
        let _ = crate::categories::delete_category(&current);
        self.category_map = crate::categories::load();
        self.categories = { let mut c = vec!["All".to_string()]; c.extend(self.category_map.categories()); c };
        self.recompute_filter();
    }

    pub fn confirm_input(&mut self) {
        let Some(mode) = &self.input_mode else { return };
        let name = self.input_buffer.trim().to_string();
        if name.is_empty() { self.cancel_input(); return; }

        match mode {
            InputMode::AssignCategory => {
                if let Some(pkg_name) = self.selected_package().map(|p| p.name.clone()) {
                    self.category_map.lookup.insert(pkg_name.clone(), name.clone());
                    if !self.category_map.order.contains(&name) {
                        self.category_map.order.push(name.clone());
                        self.category_map.order.sort();
                    }
                    let _ = crate::categories::assign_package(&pkg_name, &name);
                    self.categories = { let mut c = vec!["All".to_string()]; c.extend(self.category_map.categories()); c };
                    self.recompute_filter();
                }
            }
            InputMode::RenameCategory => {
                let current = self.categories[self.selected_category].clone();
                let _ = crate::categories::rename_category(&current, &name);
                self.category_map = crate::categories::load();
                self.categories = { let mut c = vec!["All".to_string()]; c.extend(self.category_map.categories()); c };
                self.recompute_filter();
            }
        }
        self.cancel_input();
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = None;
        self.input_buffer.clear();
    }

    pub fn selected_package(&self) -> Option<&Package> {
        self.filtered
            .get(self.package_state)
            .map(|&i| &self.all_packages[i])
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if self.input_mode.is_some() {
            match key.code {
                KeyCode::Esc => self.cancel_input(),
                KeyCode::Enter => self.confirm_input(),
                KeyCode::Backspace => { self.input_buffer.pop(); }
                KeyCode::Char(c) => self.input_buffer.push(c),
                _ => {}
            }
            return;
        }
        if self.show_help {
            if matches!(key.code, KeyCode::Char('?') | KeyCode::Esc) {
                self.show_help = false;
            }
            return;
        }
        match self.focus {
            Focus::Filter => self.handle_filter_keys(key),
            _ => self.handle_nav_keys(key),
        }
    }

    fn handle_filter_keys(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.filter_text.clear();
                self.focus = Focus::Packages;
                self.recompute_filter();
            }
            KeyCode::Enter => {
                self.focus = Focus::Packages;
            }
            KeyCode::Backspace => {
                self.filter_text.pop();
                self.recompute_filter();
            }
            KeyCode::Char('/') => {
                self.filter_text.clear();
                self.recompute_filter();
            }
            KeyCode::Char(c) => {
                self.filter_text.push(c);
                self.recompute_filter();
            }
            _ => {}
        }
    }

    fn handle_nav_keys(&mut self, key: crossterm::event::KeyEvent) {
        if key.code != KeyCode::Char('g') {
            self.pending_g = None;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Char('g') => {
                let now = Instant::now();
                let is_double_tap = self
                    .pending_g
                    .is_some_and(|t| now.duration_since(t) < Duration::from_millis(500));
                if is_double_tap {
                    self.pending_g = None;
                    self.jump_top();
                } else {
                    self.pending_g = Some(now);
                }
            }
            KeyCode::Char('G') => self.jump_bottom(),
            KeyCode::Char('.') => self.toggle_dependencies(),
            KeyCode::Char('o') => self.toggle_orphans_only(),
            KeyCode::Char('/') => self.focus = Focus::Filter,
            KeyCode::Char('m') => self.start_assign_category(),
            KeyCode::Char('r') if self.focus == Focus::Categories => self.start_rename_category(),
            KeyCode::Char('d') if self.focus == Focus::Categories => self.delete_selected_category(),
            KeyCode::Char('l') => {
                self.focus = match self.focus {
                    Focus::Categories => Focus::Packages,
                    Focus::Packages => Focus::Detail,
                    Focus::Detail => Focus::Categories,
                    Focus::Filter => Focus::Packages,
                }
            }
            KeyCode::Char('h') => {
                self.focus = match self.focus {
                    Focus::Categories => Focus::Detail,
                    Focus::Packages => Focus::Categories,
                    Focus::Detail => Focus::Packages,
                    Focus::Filter => Focus::Categories,
                }
            }
            KeyCode::Char('j') => match self.focus {
                Focus::Categories => self.move_category(1),
                Focus::Packages => self.move_package(1),
                Focus::Detail => self.scroll_detail(1),
                Focus::Filter => {}
            },
            KeyCode::Char('k') => match self.focus {
                Focus::Categories => self.move_category(-1),
                Focus::Packages => self.move_package(-1),
                Focus::Detail => self.scroll_detail(-1),
                Focus::Filter => {}
            },

            KeyCode::Char('R') => {
                self.category_map = crate::categories::load();

                let mut new_categories = vec!["All".to_string()];
                new_categories.extend(self.category_map.categories());
                self.categories = new_categories;

                if self.selected_category >= self.categories.len() {
                    self.selected_category = 0;
                }
                self.recompute_filter();
            }
            KeyCode::Char('s') => self.cycle_sort(),
            _ => {}
        }
    }
}
