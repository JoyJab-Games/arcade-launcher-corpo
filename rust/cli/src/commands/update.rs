use std::error::Error;

use arcade_core::{GameConfig, GameStore, ProcessLock, ResolvedGame};

use super::metadata;

/// Re-provisions an already-installed game using exactly what's on record
/// (same source, source_ref, branch) and refreshes its tags/preview image
/// the same way `install` does. Also what `arcade install` points an admin
/// at when the chosen name already has a valid manifest, so a re-run of
/// `install` never has to guess whether that's a conflict or a refresh.
/// Not a way to change what's installed under a name — that's `arcade
/// remove` + `arcade install` again.
pub fn run(store: &GameStore, name: &str) -> Result<(), Box<dyn Error>> {
    let _lock = ProcessLock::acquire(arcade_core::data_dir().join("arcade.lock"))?;

    let existing = store
        .get(name)
        .ok_or_else(|| format!("no game named '{name}' found"))?;

    let sources = super::available_sources();
    let source = sources
        .iter()
        .find(|s| s.id() == existing.source)
        .ok_or_else(|| format!("unknown source '{}' for '{name}'", existing.source))?
        .as_ref();

    let resolved = ResolvedGame {
        suggested_name: existing.name.clone(),
        source_ref: existing.source_ref.clone(),
        branch: existing.branch.clone(),
    };

    let fetched = source.fetch_metadata(&resolved).unwrap_or_default();

    let dest = arcade_core::game_data_dir(name);
    source.provision(&resolved, &dest)?;

    let game_dir = arcade_core::game_dir(name);
    let tags = metadata::resolve_tags(fetched.tags)?;
    let image_path = metadata::resolve_image(fetched.image_url, &game_dir)?;

    // A re-fetch that finds nothing (network hiccup, schema drift) keeps
    // whatever exec/description was already on record rather than
    // clobbering a working value with None — same non-destructive-on-
    // failure approach as tags/image_path above.
    store.save(&GameConfig {
        exec: fetched.exec.or_else(|| existing.exec.clone()),
        description: fetched.description.or_else(|| existing.description.clone()),
        tags,
        image_path,
        ..existing
    })?;

    println!("Updated '{name}'.");
    Ok(())
}
