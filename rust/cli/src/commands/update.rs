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
    // whatever exec/description/proton was already on record rather than
    // clobbering a working value with None/false — same non-destructive-
    // on-failure approach as tags/image_path above.
    let proton = fetched.proton.unwrap_or(existing.proton);
    let proton_version = resolve_proton_version(proton, existing.proton_version.clone())?;

    store.save(&GameConfig {
        exec: fetched.exec.or_else(|| existing.exec.clone()),
        description: fetched.description.or_else(|| existing.description.clone()),
        tags,
        image_path,
        proton,
        proton_version,
        ..existing
    })?;

    println!("Updated '{name}'.");
    Ok(())
}

/// Ensures `version`'s Proton build is on disk before `update` finishes —
/// same "fail loudly now, not at launch" approach `provision()` above takes
/// for the game's own files. A build that's already cached is only
/// re-fetched if the admin opts in: Proton Experimental is a rolling
/// target under a fixed AppID (see `arcade_core::sources::proton`), so a
/// bare `arcade update <name>` shouldn't silently swap it out from under a
/// working game just because the admin wanted to refresh a preview image.
fn resolve_proton_version(
    proton: bool,
    existing_version: Option<String>,
) -> Result<Option<String>, Box<dyn Error>> {
    if !proton {
        return Ok(None);
    }
    let version = existing_version.unwrap_or_else(|| arcade_core::sources::proton::DEFAULT_VERSION.to_string());
    let root = arcade_core::proton_dir();

    if arcade_core::sources::proton::is_provisioned(&root, &version) {
        if metadata::confirm(&format!("Proton build '{version}' is already downloaded — refresh it too?"))? {
            arcade_core::sources::proton::provision(&root, &version)?;
        }
    } else {
        println!("Fetching Proton ({version})...");
        arcade_core::sources::proton::provision(&root, &version)?;
    }

    Ok(Some(version))
}
