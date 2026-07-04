mod categories;
mod pacman;

use categories::CategoryMap;
use pacman::Package;

fn main() {
    let mut map = CategoryMap::default();
    map.lookup.insert("bat".to_string(), "CLI Tools".to_string());

    let pkg = Package {
        name: "bat".to_string(),
        version: "0.26.1-1".to_string(),
        description: "A cat clone with syntax highlighting".to_string(),
    };

    println!("Package: {}", pkg.name);
    println!("Category: {}", map.get(&pkg.name));
}
