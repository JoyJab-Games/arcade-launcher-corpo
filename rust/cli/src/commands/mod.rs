pub mod install;
pub mod list;
pub mod metadata;
pub mod power;
pub mod release;
pub mod remove;
pub mod update;

use arcade_core::sources::steam::SteamSource;
use arcade_core::GameSource;

/// Registered sources. V1: just Steam. Adding a new source means adding it
/// here — nothing else about install/update changes (see the GameSource
/// trait / arcade-launcher Phase 1 pitch). Shared by `install` (interactive
/// pick) and `update` (looks one up by its stored `source` id).
pub fn available_sources() -> Vec<Box<dyn GameSource>> {
    vec![Box::new(SteamSource)]
}
