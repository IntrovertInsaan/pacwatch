use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawConfig {
    pub categories: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default)]
pub struct CategoryMap {
    pub lookup: HashMap<String, String>,
    pub order: Vec<String>,
}

impl CategoryMap {
    pub fn get(&self, package_name: &str) -> &str {
        self.lookup
            .get(package_name)
            .map(|s| s.as_str())
            .unwrap_or("Uncategorized")
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats = self.order.clone();
        cats.push("Uncategorized".to_string());
        cats
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("pacwatch/categories.toml")
}

pub fn load() -> CategoryMap {
    let path = config_path();
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let parsed: RawConfig = toml::from_str(&raw).unwrap_or_default();

    let mut map = CategoryMap::default();
    for (category, packages) in &parsed.categories {
        map.order.push(category.clone());
        for pkg in packages {
            map.lookup.insert(pkg.clone(), category.clone());
        }
    }
    map.order.sort();
    map
}

const DEFAULT_CATEGORIES_TOML: &str = include_str!("../categories.default.toml");

pub fn ensure_default_config() -> std::io::Result<()> {
    let path = config_path();
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, DEFAULT_CATEGORIES_TOML)?;
    Ok(())
}

pub fn reset_config() -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, DEFAULT_CATEGORIES_TOML)
}
