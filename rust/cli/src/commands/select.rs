use std::error::Error;

use dialoguer::MultiSelect;

use arcade_core::GameStore;

/// Interactive multi-toggle version of `release`/`unrelease` — for setting
/// up (or reshuffling) which games are released in one pass, rather than
/// one `arcade release <name>` call per game. `release`/`unrelease`
/// themselves stay as they are (explicit, scriptable, no interaction
/// needed) - this is purely an additional, friendlier way to reach the
/// same `released_for_players` flag `GameStore::set_released` already
/// owns.
pub fn run(store: &GameStore) -> Result<(), Box<dyn Error>> {
    let games = store.list()?;
    if games.is_empty() {
        println!("No games installed.");
        return Ok(());
    }

    let labels: Vec<&str> = games.iter().map(|g| g.name.as_str()).collect();
    let defaults: Vec<bool> = games.iter().map(|g| g.released_for_players).collect();

    let chosen = MultiSelect::new()
        .with_prompt("Space to toggle, enter to confirm which games are released to players")
        .items(&labels)
        .defaults(&defaults)
        .interact()?;

    for (i, game) in games.iter().enumerate() {
        let released = chosen.contains(&i);
        if released != game.released_for_players {
            store.set_released(&game.name, released)?;
            println!("{} '{}'.", if released { "Released" } else { "Unreleased" }, game.name);
        }
    }

    Ok(())
}
