pub mod game_config;
pub mod lock;
pub mod power;
pub mod sources;

pub use game_config::{GameConfig, GameStore, StoreError};
pub use lock::ProcessLock;
pub use sources::{GameSource, ResolvedGame};

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

/// Where a game's provisioned files live — shared by `arcade install`
/// (creates it) and `arcade remove` (deletes it), so the two can't drift.
pub fn game_dir(name: &str) -> std::path::PathBuf {
    data_dir().join("games").join(name)
}
