use std::error::Error;

use arcade_core::GameStore;

pub fn run(store: &GameStore) -> Result<(), Box<dyn Error>> {
    let games = store.list()?;
    if games.is_empty() {
        println!("No games installed.");
        return Ok(());
    }
    for game in games {
        let status = if game.released_for_players {
            "released"
        } else {
            "hidden"
        };
        println!("{}\t{}\t{}\t{}", game.name, game.source, game.source_ref, status);
    }
    Ok(())
}
