use std::fs;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub installed_size: u64,
    pub url: String,
    pub description: String,
    pub licenses: Vec<String>,
    pub groups: Vec<String>,
    pub provides: Vec<String>,
    pub depends: Vec<String>,
    pub optdepends: Vec<String>,
    pub packager: String,
    pub required_by: Vec<String>,
    pub optional_for: Vec<String>,
    pub build_date: i64,
    pub install_date: i64,
    pub install_reason: String,
    pub validated_by: String,
    pub files: Vec<String>,
}

pub fn human_size(size: u64) -> String {
    if size < 1024 { format!("{} B", size) }
    else if size < 1024 * 1024 { format!("{:.2} KB", size as f64 / 1024.0) }
    else { format!("{:.2} MB", size as f64 / (1024.0 * 1024.0)) }
}

fn parse_desc(raw: &str) -> Package {
    let mut pkg = Package::default();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.starts_with('%') { continue; }
        let field = line.trim_matches('%');

        let mut values = Vec::new();
        while let Some(next) = lines.peek() {
            if next.is_empty() { lines.next(); break; }
            values.push(lines.next().unwrap().to_string());
        }

        match field {
            "NAME" => pkg.name = values.first().cloned().unwrap_or_default(),
            "VERSION" => pkg.version = values.first().cloned().unwrap_or_default(),
            "ARCH" => pkg.architecture = values.first().cloned().unwrap_or_default(),
            "ISIZE" | "SIZE" => pkg.installed_size = values.first().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0),
            "URL" => pkg.url = values.first().cloned().unwrap_or_default(),
            "DESC" => pkg.description = values.first().cloned().unwrap_or_default(),
            "LICENSE" => pkg.licenses = values,
            "GROUPS" => pkg.groups = values,
            "PROVIDES" => pkg.provides = values,
            "DEPENDS" => pkg.depends = values,
            "OPTDEPENDS" => pkg.optdepends = values,
            "PACKAGER" => pkg.packager = values.first().cloned().unwrap_or_default(),
            "BUILDDATE" => pkg.build_date = values.first().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            "INSTALLDATE" => pkg.install_date = values.first().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            "REASON" => pkg.install_reason = values.first().cloned().unwrap_or_default(),
            "VALIDATEDBY" => pkg.validated_by = values.first().cloned().unwrap_or_default(),
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
    compute_reverse_deps(&mut packages); // <--- Add this!
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    packages
}

pub fn compute_reverse_deps(packages: &mut [Package]) {
    let name_to_idx: HashMap<String, usize> = packages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), i))
        .collect();

    for p in packages.iter() {
        for dep in &p.depends {
            let base_name = dep.split(['=', '<', '>']).next().unwrap_or(dep).trim();
            if let Some(&idx) = name_to_idx.get(base_name) {
                packages[idx].required_by.push(p.name.clone());
            }
        }
        for dep in &p.optdepends {
            let base_name = dep.split(':').next().unwrap_or(dep).trim();
            if let Some(&idx) = name_to_idx.get(base_name) {
                packages[idx].optional_for.push(p.name.clone());
            }
        }
    }
}
