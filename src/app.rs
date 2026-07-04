use crate::categories::CategoryMap;
use crate::pacman::Package;

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
    pub package_state: usize,
    pub focus: Focus,
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
            package_state: 0,
            focus: Focus::Categories,
        };
        app.recompute_filter();
        app
    }

    pub fn recompute_filter(&mut self) {
        let cat = &self.categories[self.selected_category];
        self.filtered = self.all_packages.iter().enumerate()
            .filter(|(_, p)| cat == "All" || self.category_map.get(&p.name) == cat)
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
}
