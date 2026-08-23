use std::path::Path;

use crate::game_config::GameConfig;

pub mod steam;
pub mod steamcmd_session;

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
    /// Source-specific release channel (e.g. a Steam beta branch) — most
    /// sources leave this `None`. Carried here rather than bolted onto
    /// `source_ref` so `provision`/`check_update` can use it without
    /// needing to parse it back out.
    pub branch: Option<String>,
}

/// Best-effort display info for a resolved game, fetched by the source
/// itself (e.g. Steam's public appdetails endpoint) before the admin is
/// asked to confirm a name. Any field may come back empty — that's not an
/// error, just "this source has nothing to offer here": `arcade
/// install`/`update` fall back to the AppID/source_ref as the suggested
/// name, and to asking the admin to type tags or paste an image URL
/// manually, in that case.
#[derive(Debug, Clone, Default)]
pub struct GameMetadata {
    /// A human-readable name to suggest at the "save as which name?"
    /// prompt — never what actually gets stored as GameConfig.name, which
    /// stays the admin's own choice (see GameConfig's doc comment).
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    /// Path to the Linux executable, relative to the game's `game_data/`
    /// folder — same shape as `GameConfig.exec`, which this fills in
    /// directly at install/update time.
    pub exec: Option<String>,
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

    /// Best-effort tags + preview image for `game` (see GameMetadata's doc
    /// comment). Default: nothing — a source that has no metadata of its
    /// own (e.g. a future SD-card source) just doesn't override this, and
    /// the CLI falls straight through to asking the admin manually.
    fn fetch_metadata(&self, _game: &ResolvedGame) -> std::io::Result<GameMetadata> {
        Ok(GameMetadata::default())
    }
}
