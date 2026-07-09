use std::fs;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstallReason {
    #[default]
    Explicit,
    Dependency,
}

impl InstallReason {
    pub fn is_explicit(self) -> bool {
        matches!(self, InstallReason::Explicit)
    }

    pub fn label(self) -> &'static str {
        match self {
            InstallReason::Explicit => "Explicitly installed",
            InstallReason::Dependency => "Installed as a dependency",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub url: String,
    pub packager: String,
    pub licenses: Vec<String>,
    pub validated_by: String,
    pub install_reason: InstallReason,
    pub installed_size: u64,
    pub install_date: i64,
    pub build_date: i64,
    pub groups: Vec<String>,
    pub provides: Vec<String>,
    pub depends: Vec<String>,
    pub optdepends: Vec<String>,
    pub required_by: Vec<String>,
    pub optional_for: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub has_install_script: bool,
    pub files: Vec<String>,
}

impl Package {
    pub fn is_orphan(&self) -> bool {
        !self.install_reason.is_explicit() && self.required_by.is_empty()
    }
}

fn parse_desc(raw: &str) -> Package {
    let mut pkg = Package::default();

    let mut lines = raw.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.starts_with('%') {
            continue;
        }
        let field = line.trim_matches('%');
        let mut values = Vec::new();
        while let Some(next) = lines.peek() {
            if next.is_empty() {
                lines.next();
                break;
            }
            values.push(lines.next().unwrap().to_string());
        }

        match field {
            "NAME" => pkg.name = values.into_iter().next().unwrap_or_default(),
            "VERSION" => pkg.version = values.into_iter().next().unwrap_or_default(),
            "DESC" => pkg.description = values.into_iter().next().unwrap_or_default(),
            "ARCH" => pkg.architecture = values.into_iter().next().unwrap_or_default(),
            "URL" => pkg.url = values.into_iter().next().unwrap_or_default(),
            "LICENSE" => pkg.licenses = values,
            "GROUPS" => pkg.groups = values,
            "PROVIDES" => pkg.provides = values,
            "DEPENDS" => pkg.depends = values,
            "OPTDEPENDS" => pkg.optdepends = values,
            "CONFLICTS" => pkg.conflicts = values,
            "REPLACES" => pkg.replaces = values,
            "PACKAGER" => pkg.packager = values.into_iter().next().unwrap_or_default(),
            "BUILDDATE" => {
                pkg.build_date = values.into_iter().next().and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            "INSTALLDATE" => {
                pkg.install_date = values.into_iter().next().and_then(|v| v.parse().ok()).unwrap_or(0);
            }
            "REASON" => {
                let r = values.into_iter().next().unwrap_or_default();
                if r == "1" {
                    pkg.install_reason = InstallReason::Dependency;
                }
            }
            "VALIDATION" => pkg.validated_by = values.join(", "),
            "SIZE" => {
                pkg.installed_size = values
                    .into_iter()
                    .next()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);
            }
            _ => {}
        }
    }
    pkg
}

fn parse_files(raw: &str) -> Vec<String> {
    raw.lines()
        .skip(1)
        .filter(|l| !l.is_empty())
        .map(|l| format!("/{}", l))
        .collect()
}

pub fn load_installed_packages() -> std::io::Result<Vec<Package>> {
    load_installed_packages_from(Path::new("/var/lib/pacman/local"))
}

pub fn load_installed_packages_from(base: &Path) -> std::io::Result<Vec<Package>> {
    let mut packages = Vec::new();
    if !base.exists() {
        return Ok(packages);
    }

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let desc_path = path.join("desc");
        if !desc_path.exists() {
            continue;
        }

        let raw = fs::read_to_string(&desc_path)?;
        let mut pkg = parse_desc(&raw);
        pkg.has_install_script = path.join("install").exists();

        let files_path = path.join("files");
        if let Ok(raw_files) = fs::read_to_string(&files_path) {
            pkg.files = parse_files(&raw_files);
        }

        packages.push(pkg);
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));
    compute_reverse_deps(&mut packages);
    Ok(packages)
}

fn dep_base_name(dep: &str) -> &str {
    dep.split(['=', '<', '>']).next().unwrap_or(dep).trim()
}

fn optdep_base_name(dep: &str) -> &str {
    dep.split(':').next().unwrap_or(dep).trim()
}

fn compute_reverse_deps(packages: &mut [Package]) {
    let name_to_idx: HashMap<String, usize> = packages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.name.clone(), i))
        .collect();

    let mut required_by: HashMap<usize, Vec<String>> = HashMap::new();
    let mut optional_for: HashMap<usize, Vec<String>> = HashMap::new();

    for p in packages.iter() {
        for dep in &p.depends {
            if let Some(&idx) = name_to_idx.get(dep_base_name(dep)) {
                required_by.entry(idx).or_default().push(p.name.clone());
            }
        }
        for dep in &p.optdepends {
            if let Some(&idx) = name_to_idx.get(optdep_base_name(dep)) {
                optional_for.entry(idx).or_default().push(p.name.clone());
            }
        }
    }

    for (idx, mut names) in required_by {
        names.sort();
        packages[idx].required_by = names;
    }
    for (idx, mut names) in optional_for {
        names.sort();
        packages[idx].optional_for = names;
    }
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, UNITS[unit])
}

pub fn format_epoch(epoch: i64) -> String {
    if epoch == 0 {
        return "Unknown".to_string();
    }
    match chrono::DateTime::from_timestamp(epoch, 0) {
        Some(utc) => {
            let local: chrono::DateTime<chrono::Local> = utc.into();
            local.format("%a %d %b %Y %I:%M:%S %p %Z").to_string()
        }
        None => "Unknown".to_string(),
    }
}
