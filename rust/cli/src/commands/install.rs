use std::error::Error;
use std::io::{self, Write};

use arcade_core::sources::steam::SteamSource;
use arcade_core::{GameConfig, GameSource, GameStore, ProcessLock};

/// Registered sources. V1: just Steam. Adding a new source means adding it
/// here — nothing else about the install flow below changes (see the
/// GameSource trait / arcade-launcher Phase 1 pitch).
fn available_sources() -> Vec<Box<dyn GameSource>> {
    vec![Box::new(SteamSource)]
}

pub fn run(store: &GameStore) -> Result<(), Box<dyn Error>> {
    let _lock = ProcessLock::acquire(arcade_core::data_dir().join("arcade.lock"))?;

    let sources = available_sources();
    let source_idx = if sources.len() == 1 {
        0
    } else {
        pick_source(&sources)?
    };
    let source = sources[source_idx].as_ref();

    let resolved = source.resolve_install_target()?;

    let chosen = prompt(&format!(
        "Save as which name? [{}]: ",
        resolved.suggested_name
    ))?;
    let name = if chosen.is_empty() {
        resolved.suggested_name.clone()
    } else {
        chosen
    };

    if store.get(&name)?.is_some() {
        return Err(format!("a game named '{name}' already exists").into());
    }

    let dest = arcade_core::game_dir(&name);
    source.provision(&resolved, &dest)?;

    store.add(GameConfig {
        name: name.clone(),
        source: source.id().to_string(),
        source_ref: resolved.source_ref,
        exec: None,
        proton: false,
        prefix_path: None,
        released_for_players: false,
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
