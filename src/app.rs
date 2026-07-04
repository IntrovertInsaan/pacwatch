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
    /// Timestamp of a lone 'g' press, waiting to see if 'g' follows within
    /// the double-tap window to form "gg" (jump to top). None between taps.
    pub pending_g: Option<Instant>,
    /// Toggled with '.', yazi-style. Off by default: only explicitly-installed
    /// packages show. On: dependency-tail packages are added in too (dimmed),
    /// additive rather than a separate mode.
    pub show_dependencies: bool,
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
            pending_g: None,
            show_dependencies: false,
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
                let reason_ok = self.show_dependencies || p.install_reason == "Explicitly installed";
                cat_ok && text_ok && reason_ok
            })
            .map(|(i, _)| i)
            .collect();
    }

    pub fn move_package(&mut self, delta: i32) {
        if self.filtered.is_empty() { return; }
        let next = (self.package_state as i32 + delta).clamp(0, self.filtered.len() as i32 - 1);
        self.package_state = next as usize;
        self.detail_scroll = 0;
    }

    pub fn scroll_detail(&mut self, delta: i32) {
        let next = self.detail_scroll as i32 + delta;
        self.detail_scroll = next.max(0) as u16;
        // Upper bound is clamped in ui.rs against the actual rendered content
        // height, since that's the only place that knows both.
    }

    /// vim 'gg' -- jump to the top of whichever pane has focus.
    pub fn jump_top(&mut self) {
        match self.focus {
            Focus::Categories => {
                self.selected_category = 0;
                self.package_state = 0;
                self.detail_scroll = 0;
                self.recompute_filter();
            }
            Focus::Packages => {
                self.package_state = 0;
                self.detail_scroll = 0;
            }
            Focus::Detail => self.detail_scroll = 0,
            Focus::Filter => {}
        }
    }

    /// vim 'G' -- jump to the bottom of whichever pane has focus.
    pub fn jump_bottom(&mut self) {
        match self.focus {
            Focus::Categories => {
                self.selected_category = self.categories.len().saturating_sub(1);
                self.package_state = 0;
                self.detail_scroll = 0;
                self.recompute_filter();
            }
            Focus::Packages => {
                self.package_state = self.filtered.len().saturating_sub(1);
                self.detail_scroll = 0;
            }
            // ui.rs already clamps detail_scroll to the real content height on
            // render, so an oversized value here just resolves to "last line".
            Focus::Detail => self.detail_scroll = u16::MAX,
            Focus::Filter => {}
        }
    }

    pub fn move_category(&mut self, delta: i32) {
        let len = self.categories.len() as i32;
        if len == 0 { return; }
        let mut next = self.selected_category as i32 + delta;
        next = next.clamp(0, len - 1);
        self.selected_category = next as usize;
        self.recompute_filter();
    }

    /// yazi-style '.' toggle: show/hide dependency-tail packages.
    pub fn toggle_dependencies(&mut self) {
        self.show_dependencies = !self.show_dependencies;
        self.recompute_filter();
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
        // Any key other than a second 'g' cancels a pending "waiting for gg" state.
        if key.code != KeyCode::Char('g') {
            self.pending_g = None;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
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
            KeyCode::Char('/') => self.focus = Focus::Filter,
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

            KeyCode::Char('r') => {
                self.category_map = crate::categories::load();

                let mut new_categories = vec!["All".to_string()];
                new_categories.extend(self.category_map.categories());
                self.categories = new_categories;

                if self.selected_category >= self.categories.len() {
                    self.selected_category = 0;
                }
                self.recompute_filter();
            }
            _ => {}
        }
    }
}
