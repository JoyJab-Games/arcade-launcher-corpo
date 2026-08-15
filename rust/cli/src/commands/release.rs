use std::error::Error;

use arcade_core::GameStore;

pub fn run(store: &GameStore, name: &str, released: bool) -> Result<(), Box<dyn Error>> {
    store.set_released(name, released)?;
    println!(
        "{} '{name}'.",
        if released { "Released" } else { "Unreleased" }
    );
    Ok(())
}
