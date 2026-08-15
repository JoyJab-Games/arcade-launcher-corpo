use std::error::Error;

use arcade_core::GameStore;

pub fn run(store: &GameStore, name: &str) -> Result<(), Box<dyn Error>> {
    store.remove(name)?;

    let dir = arcade_core::game_dir(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }

    println!("Removed '{name}'.");
    Ok(())
}
