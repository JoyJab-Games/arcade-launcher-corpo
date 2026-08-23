use std::error::Error;

use arcade_core::GameStore;

pub fn run(store: &GameStore, name: &str) -> Result<(), Box<dyn Error>> {
    // GameStore::remove deletes the game's whole folder (manifest, preview
    // image, and provisioned files together) in one step — see its doc
    // comment on why that can't drift apart into two separate deletions.
    store.remove(name)?;
    println!("Removed '{name}'.");
    Ok(())
}
