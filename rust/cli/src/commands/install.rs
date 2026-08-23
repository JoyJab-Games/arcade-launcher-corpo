use std::error::Error;
use std::io::{self, Write};

use arcade_core::{GameConfig, GameSource, GameStore, ProcessLock};

use super::metadata;

pub fn run(store: &GameStore) -> Result<(), Box<dyn Error>> {
    let _lock = ProcessLock::acquire(arcade_core::data_dir().join("arcade.lock"))?;

    let sources = super::available_sources();
    let source_idx = if sources.len() == 1 {
        0
    } else {
        pick_source(&sources)?
    };
    let source = sources[source_idx].as_ref();

    let resolved = source.resolve_install_target()?;

    // Fetched before the name prompt so a source-provided display name
    // (e.g. Steam's actual game title) can be offered as the default —
    // note this only affects the prompt's suggestion, never what actually
    // gets saved as GameConfig.name (see its doc comment: that stays the
    // admin's own stable choice).
    let fetched = source.fetch_metadata(&resolved).unwrap_or_default();
    if let Some(desc) = &fetched.description {
        println!("{}", desc);
    }

    let suggested_name = fetched.name.clone().unwrap_or_else(|| resolved.suggested_name.clone());
    let chosen = prompt(&format!("Save as which name? [{suggested_name}]: "))?;
    let name = if chosen.is_empty() { suggested_name } else { chosen };

    // A folder can exist here with no valid manifest (a leftover from a
    // previous failed/aborted install) — GameStore treats that the same as
    // "not installed" (see its doc comment), so only a *valid* existing
    // manifest is a real conflict worth stopping for.
    if store.get(&name).is_some() {
        return Err(format!(
            "a game named '{name}' already exists — use `arcade update {name}` to refresh it"
        )
        .into());
    }

    let dest = arcade_core::game_data_dir(&name);
    source.provision(&resolved, &dest)?;

    let game_dir = arcade_core::game_dir(&name);
    let tags = metadata::resolve_tags(fetched.tags)?;
    let image_path = metadata::resolve_image(fetched.image_url, &game_dir)?;

    store.save(&GameConfig {
        name: name.clone(),
        source: source.id().to_string(),
        source_ref: resolved.source_ref,
        branch: resolved.branch,
        exec: fetched.exec,
        proton: false,
        prefix_path: None,
        released_for_players: false,
        description: fetched.description,
        tags,
        image_path,
    })?;

    println!("Installed '{name}' (not yet released to players — see `arcade release {name}`).");
    Ok(())
}

fn pick_source(sources: &[Box<dyn GameSource>]) -> Result<usize, Box<dyn Error>> {
    println!("Available sources:");
    for (i, s) in sources.iter().enumerate() {
        println!("  {}) {}", i + 1, s.id());
    }
    let choice = prompt("Source: ")?;
    let idx: usize = choice.parse().map_err(|_| "invalid choice")?;
    if idx == 0 || idx > sources.len() {
        return Err("invalid choice".into());
    }
    Ok(idx - 1)
}

fn prompt(label: &str) -> io::Result<String> {
    print!("{label}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}
