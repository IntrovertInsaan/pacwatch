mod categories;
mod pacman;

use pacman::Package;

fn main() {
    categories::ensure_default_config().expect("Failed to create default config");
    let map = categories::load();

    let pkg = Package {
        name: "bat".to_string(),
        version: "0.26.1-1".to_string(),
        description: "A cat clone with syntax highlighting".to_string(),
    };

    println!("Package: {}", pkg.name);
    println!("Category: {}", map.get(&pkg.name));
}
