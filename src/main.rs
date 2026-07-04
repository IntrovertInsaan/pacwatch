mod categories;
mod pacman;

fn main() {
    categories::ensure_default_config().expect("Failed to create default config");
    let map = categories::load();

    // Using the new native loader
    let pkgs = pacman::load_installed_packages();
    println!("Found {} installed packages.", pkgs.len());

    // Displaying the first 5 packages with their info
    for pkg in pkgs.iter().take(5) {
        println!("--------------------------------");
        println!("Name:        {}", pkg.name);
        println!("Version:     {}", pkg.version);
        println!("Description: {}", pkg.description);
        println!("Category:    {}", map.get(&pkg.name));
    }
}
