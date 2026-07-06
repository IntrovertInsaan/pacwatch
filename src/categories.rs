use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawConfig {
    #[serde(default)]
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
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pacwatch")
        .join("categories.toml")
}

pub fn load() -> CategoryMap {
    let path = config_path();

    if !path.exists() {
        let _ = ensure_default_config();
    }

    let raw = fs::read_to_string(&path).expect("Failed to read categories.toml");
    let parsed: RawConfig = toml::from_str(&raw).expect("Failed to parse categories.toml");

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

use toml_edit::{DocumentMut, Array, value};

pub fn assign_package(package: &str, category: &str) -> std::io::Result<()> {
    let path = config_path();
    let text = fs::read_to_string(&path)?;
    let mut doc: DocumentMut = text.parse().expect("Failed to parse categories.toml");

    if let Some(categories) = doc["categories"].as_table_mut() {
        for (_, arr) in categories.iter_mut() {
            if let Some(arr) = arr.as_array_mut() {
                arr.retain(|v| v.as_str() != Some(package));
            }
        }
    }

    let categories = doc["categories"].or_insert(toml_edit::table()).as_table_mut().unwrap();
    if categories.get(category).is_none() {
        categories[category] = value(Array::new());
    }
    categories[category].as_array_mut().unwrap().push(package);

    fs::write(&path, doc.to_string())
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
