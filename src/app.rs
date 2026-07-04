use crate::categories::CategoryMap;
use crate::pacman::Package;
use ratatui::widgets::ListState;
use crossterm::event::KeyCode;

#[derive(PartialEq, Eq)]
pub enum Focus {
    Categories,
    Packages,
    Filter,
}

pub struct App {
    pub all_packages: Vec<Package>,
    pub category_map: CategoryMap,
    pub categories: Vec<String>,
    pub selected_category: usize,
    pub filtered: Vec<usize>,
    pub filter_text: String,
    pub package_state: usize,
    pub focus: Focus,
    pub cat_state: ListState,
    pub pkg_state: ListState,
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
            focus: Focus::Categories,
            cat_state: ListState::default(),
            pkg_state: ListState::default(),
        };
        app.recompute_filter();
        app
    }

    pub fn recompute_filter(&mut self) {
        let cat = &self.categories[self.selected_category];
        let needle = self.filter_text.to_lowercase();

        self.filtered = self.all_packages.iter().enumerate()
            .filter(|(_, p)| {
                let cat_ok = cat == "All" || self.category_map.get(&p.name) == cat;
                let text_ok = needle.is_empty() || p.name.to_lowercase().contains(&needle);
                cat_ok && text_ok
            })
            .map(|(i, _)| i)
            .collect();
    }

    pub fn move_package(&mut self, delta: i32) {
        if self.filtered.is_empty() { return; }
        let next = (self.package_state as i32 + delta).clamp(0, self.filtered.len() as i32 - 1);
        self.package_state = next as usize;
    }

    pub fn move_category(&mut self, delta: i32) {
        let len = self.categories.len() as i32;
        if len == 0 { return; }
        let mut next = self.selected_category as i32 + delta;
        next = next.clamp(0, len - 1);
        self.selected_category = next as usize;
        self.recompute_filter();
    }

    pub fn move_selection(&mut self, delta: i32) {
        match self.focus {
            Focus::Categories => {
                let len = self.categories.len() as i32;
                let next = (self.selected_category as i32 + delta).clamp(0, len - 1);
                self.selected_category = next as usize;
                self.cat_state.select(Some(self.selected_category));
                self.recompute_filter();
            }
            Focus::Packages => {
                self.move_package(delta);
                let next = (self.package_state as i32).clamp(0, self.filtered.len() as i32 - 1);
                self.pkg_state.select(Some(next as usize));
            }
            Focus::Filter => {}
        }
    }

    pub fn selected_package(&self) -> Option<&Package> {
        self.filtered
            .get(self.package_state)
            .map(|&i| &self.all_packages[i])
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
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
            KeyCode::Backspace => {
                self.filter_text.pop();
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
        match key.code {
            KeyCode::Char('/') => self.focus = Focus::Filter,
            KeyCode::Char('h') => self.focus = Focus::Categories,
            KeyCode::Char('l') => self.focus = Focus::Packages,
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            _ => {}
        }
    }
}
