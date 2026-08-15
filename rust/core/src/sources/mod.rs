use std::path::Path;

use crate::game_config::GameConfig;

pub mod steam;

/// A game resolved from a source, before it's been given a local reference
/// name (see GameSource::resolve_install_target).
#[derive(Debug, Clone)]
pub struct ResolvedGame {
    /// Suggested local name, e.g. Steam's app title — the CLI still lets
    /// the admin confirm/override it at install time.
    pub suggested_name: String,
    /// Opaque per-source reference (Steam AppID, SD-card path, remote
    /// URL, ...) stored in the game's config afterward. Nothing outside
    /// the owning GameSource impl interprets this.
    pub source_ref: String,
}

/// One pluggable way to obtain/track games. Per the arcade-launcher
/// briefing's Phase 1 GameSource pitch: a future SD-card source or
/// online-folder-sync source must be addable as pure implementations of
/// this trait, with no changes to the CLI, GameStore, or config schema.
pub trait GameSource {
    /// Short identifier used on the CLI, e.g. "steam".
    fn id(&self) -> &'static str;

    /// Interactively resolves which game to install (prompts on
    /// stdin/stdout) — each source defines its own selection UX: Steam
    /// asks for an AppID, a future SD-card source would list detected
    /// candidate folders, a future sync source would list remote entries.
    fn resolve_install_target(&self) -> std::io::Result<ResolvedGame>;

    /// Downloads/copies/mounts whatever is needed so the game is playable
    /// locally, into `dest`.
    fn provision(&self, game: &ResolvedGame, dest: &Path) -> std::io::Result<()>;

    /// Checks whether an already-installed game has an update available.
    fn check_update(&self, game: &GameConfig) -> std::io::Result<bool>;
}
