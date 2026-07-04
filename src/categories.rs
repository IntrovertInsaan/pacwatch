use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct CategoryMap {
    pub lookup: HashMap<String, String>,
}

impl CategoryMap {
    pub fn get(&self, package_name: &str) -> &str {
        self.lookup
            .get(package_name)
            .map(|s| s.as_str())
            .unwrap_or("Uncategorized")
    }
}
