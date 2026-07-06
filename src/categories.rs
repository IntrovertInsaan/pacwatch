use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RawConfig {
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct CategoryError(pub String);

impl fmt::Display for CategoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<std::io::Error> for CategoryError {
    fn from(e: std::io::Error) -> Self {
        CategoryError(format!("I/O error: {e}"))
    }
}

impl From<toml_edit::TomlError> for CategoryError {
    fn from(e: toml_edit::TomlError) -> Self {
        CategoryError(format!("Invalid categories.toml: {e}"))
    }
}

type CatResult<T> = Result<T, CategoryError>;

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
    try_load().unwrap_or_else(|_| CategoryMap::default())
}

pub fn try_load() -> CatResult<CategoryMap> {
    let path = config_path();

    if !path.exists() {
        ensure_default_config()?;
    }

    let raw = fs::read_to_string(&path)?;
    let parsed: RawConfig = toml::from_str(&raw)
        .map_err(|e| CategoryError(format!("Invalid categories.toml: {e}")))?;

    let mut map = CategoryMap::default();
    for (category, packages) in &parsed.categories {
        map.order.push(category.clone());
        for pkg in packages {
            map.lookup.insert(pkg.clone(), category.clone());
        }
    }
    map.order.sort();
    Ok(map)
}

use toml_edit::{DocumentMut, Array, value};

pub const RESERVED_NAMES: [&str; 2] = ["All", "Uncategorized"];

pub fn is_reserved_name(name: &str) -> bool {
    RESERVED_NAMES.iter().any(|r| r.eq_ignore_ascii_case(name))
}

fn read_doc(path: &PathBuf) -> CatResult<DocumentMut> {
    let text = fs::read_to_string(path)?;
    let doc: DocumentMut = text.parse()?;
    Ok(doc)
}

pub fn assign_package(package: &str, category: &str) -> CatResult<()> {
    let path = config_path();
    let mut doc = read_doc(&path)?;

    if let Some(categories) = doc["categories"].as_table_mut() {
        for (_, arr) in categories.iter_mut() {
            if let Some(arr) = arr.as_array_mut() {
                arr.retain(|v| v.as_str() != Some(package));
            }
        }
    }

    let categories = doc["categories"].or_insert(toml_edit::table()).as_table_mut()
        .ok_or_else(|| CategoryError("categories.toml: [categories] is not a table".into()))?;
    if categories.get(category).is_none() {
        categories[category] = value(Array::new());
    }
    categories[category].as_array_mut()
        .ok_or_else(|| CategoryError(format!("categories.toml: '{category}' is not an array")))?
        .push(package);

    fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn add_category(name: &str) -> CatResult<()> {
    if is_reserved_name(name) {
        return Err(CategoryError(format!("\"{name}\" is a reserved name")));
    }
    let path = config_path();
    let mut doc = read_doc(&path)?;
    let categories = doc["categories"].or_insert(toml_edit::table()).as_table_mut()
        .ok_or_else(|| CategoryError("categories.toml: [categories] is not a table".into()))?;
    if categories.get(name).is_some() {
        return Err(CategoryError(format!("Category \"{name}\" already exists")));
    }
    categories[name] = value(Array::new());
    fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn rename_category(old: &str, new: &str) -> CatResult<()> {
    if is_reserved_name(new) {
        return Err(CategoryError(format!("\"{new}\" is a reserved name")));
    }
    let path = config_path();
    let mut doc = read_doc(&path)?;
    let categories = doc["categories"].as_table_mut()
        .ok_or_else(|| CategoryError("categories.toml has no [categories] table".into()))?;

    if old.eq_ignore_ascii_case(new) {
        return Ok(()); // no-op, not an error
    }
    if categories.get(new).is_some() {
        return Err(CategoryError(format!("Category \"{new}\" already exists")));
    }
    let entry = categories.remove(old)
        .ok_or_else(|| CategoryError(format!("Category \"{old}\" not found")))?;
    categories.insert(new, entry);
    fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn delete_category(name: &str) -> CatResult<()> {
    let path = config_path();
    let mut doc = read_doc(&path)?;
    if let Some(categories) = doc["categories"].as_table_mut() {
        categories.remove(name);
    }
    fs::write(&path, doc.to_string())?;
    Ok(())
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
