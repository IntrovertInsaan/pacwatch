use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
}

fn parse_desc(raw: &str) -> Package {
    let mut pkg = Package::default();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.starts_with('%') { continue; }
        let field = line.trim_matches('%');
        let value = lines.next().unwrap_or_default().to_string();

        match field {
            "NAME" => pkg.name = value,
            "VERSION" => pkg.version = value,
            "DESC" => pkg.description = value,
            _ => {}
        }
    }
    pkg
}

pub fn load_installed_packages() -> Vec<Package> {
    let base = Path::new("/var/lib/pacman/local");
    let mut packages = Vec::new();

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let desc_path = entry.path().join("desc");
            if desc_path.exists() {
                if let Ok(raw) = fs::read_to_string(desc_path) {
                    packages.push(parse_desc(&raw));
                }
            }
        }
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}
