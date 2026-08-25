pub mod game_config;
pub mod gamescope;
pub mod http;
pub mod input_watch;
pub mod launch;
pub mod lock;
pub mod power;
pub mod session;
pub mod sources;

pub use game_config::{GameConfig, GameStore, StoreError};
pub use launch::launch;
pub use lock::ProcessLock;
pub use sources::{GameMetadata, GameSource, ResolvedGame};

/// Where per-game config/data lives. Placeholder pending the real
/// "config storage location/format" decision (arcade-launcher open
/// questions) — override via ARCADE_DATA_DIR. Defaults to an XDG-ish path
/// for local dev/testing.
pub fn data_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("ARCADE_DATA_DIR") {
        return std::path::PathBuf::from(dir);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".local/share/arcade-launcher")
}

/// Root folder containing one subfolder per installed game (see `game_dir`).
/// `GameStore` scans this directly — there is no separate central registry
/// file, so a game's own folder is the only place its install state lives.
pub fn games_dir() -> std::path::PathBuf {
    data_dir().join("games")
}

/// A single game's own folder: `manifest.json` (its GameConfig), a fetched/
/// manually-provided preview image, and a `game_data/` subfolder for its
/// actual provisioned files (see `game_data_dir`) — kept separate from the
/// two above so a game's own install can't collide with or overwrite them.
/// Shared by every command that touches a game's files (`install`,
/// `update`, `remove`), so they can't drift apart from `GameStore`.
pub fn game_dir(name: &str) -> std::path::PathBuf {
    games_dir().join(name)
}

/// Where a game's actual provisioned files live — e.g. what `arcade install`
/// points `steamcmd`'s `+force_install_dir` at.
pub fn game_data_dir(name: &str) -> std::path::PathBuf {
    game_dir(name).join("game_data")
}

/// Root folder for cached Proton builds (see `sources::proton`) — one
/// subfolder per Steam AppID, shared across every game that pins that
/// version, so provisioning one Proton game never re-downloads it for
/// another. Deliberately separate from `games_dir()`: a Proton build isn't
/// any one game's own files. Functions in `sources::proton` take this as an
/// explicit argument rather than reading it themselves, same "explicit
/// root, testable" shape as `GameStore::open` vs `default_root()`.
pub fn proton_dir() -> std::path::PathBuf {
    data_dir().join("proton")
}

/// Where `SteamCmdSession` keeps `steamcmd`'s own login/session cache.
/// Defaults to `$XDG_RUNTIME_DIR` — tmpfs-backed and cleared at
/// logout/reboot by spec, so a cached Steam login never survives a reboot
/// without us having to clean anything up ourselves. Falls back to the
/// system temp dir (e.g. for local dev machines without a runtime dir),
/// same idea, just without the reboot guarantee. Override via
/// ARCADE_STEAMCMD_SESSION_DIR for tests/dev.
pub fn steamcmd_session_dir() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("ARCADE_STEAMCMD_SESSION_DIR") {
        return std::path::PathBuf::from(dir);
    }
    let base = std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("arcade-launcher/steamcmd")
}
