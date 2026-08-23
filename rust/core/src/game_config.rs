use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// One installed game, keyed by `name` — the stable, source-agnostic
/// reference chosen at install time (see GameSource::resolve_install_target,
/// and `arcade install`'s "save as which name?" prompt). `source`/
/// `source_ref` record where it came from but nothing else keys off them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameConfig {
    pub name: String,
    pub source: String,
    pub source_ref: String,
    /// Source-specific release channel, e.g. a Steam beta branch. Not every
    /// source uses this (same shape as `proton`/`prefix_path` below).
    #[serde(default)]
    pub branch: Option<String>,
    /// Path to the game's executable, relative to its `game_data/` folder
    /// (see `arcade_core::game_data_dir`) — e.g. "WummsenVillage.x86_64".
    #[serde(default)]
    pub exec: Option<String>,
    #[serde(default)]
    pub proton: bool,
    #[serde(default)]
    pub prefix_path: Option<String>,
    #[serde(default)]
    pub released_for_players: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Path to a preview image, relative to the game's own folder (see
    /// `arcade_core::game_dir`) — e.g. "capsule.jpg".
    #[serde(default)]
    pub image_path: Option<String>,
}

impl GameConfig {
    /// `image_path` resolved to a full filesystem path, for callers (e.g.
    /// the gdext bridge) that have no notion of `arcade_core::game_dir` and
    /// just need something they can hand straight to an image loader.
    pub fn image_abs_path(&self) -> Option<PathBuf> {
        self.image_path.as_ref().map(|path| crate::game_dir(&self.name).join(path))
    }
}

/// Folder-per-game-backed store for GameConfig entries: each game gets its
/// own `<root>/<name>/manifest.json`, alongside its preview image and
/// `game_data/` payload (see `arcade_core::game_dir`/`game_data_dir`).
///
/// There is deliberately no separate central registry file. A game's own
/// folder is the single source of truth for whether it's installed:
/// `remove` is one `rm -rf`, and a leftover folder from a failed/aborted
/// install (missing or corrupt manifest.json) is indistinguishable from
/// "not installed" — `list`/`get` just skip it — rather than a conflicting
/// state `install` has to specially detect and error on.
pub struct GameStore {
    root: PathBuf,
}

impl GameStore {
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default_root() -> PathBuf {
        crate::games_dir()
    }

    fn manifest_path(&self, name: &str) -> PathBuf {
        self.root.join(name).join("manifest.json")
    }

    /// Returns `None` both when `name` has never been installed and when
    /// its folder exists but `manifest.json` is missing/corrupt — see the
    /// struct doc comment. Callers that need to tell those apart (there's
    /// no current need to) would have to stat the folder themselves.
    pub fn get(&self, name: &str) -> Option<GameConfig> {
        let contents = fs::read_to_string(self.manifest_path(name)).ok()?;
        serde_json::from_str(&contents).ok()
    }

    /// Lists every game with a valid manifest, sorted by name. Folders with
    /// a missing/corrupt manifest are silently skipped, not reported as an
    /// error — that's the intended recovery path for a leftover folder from
    /// a failed install, not something `arcade list` should ever fail on.
    pub fn list(&self) -> io::Result<Vec<GameConfig>> {
        let mut games = Vec::new();
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(games),
            Err(e) => return Err(e),
        };

        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(game) = self.get(&name) {
                games.push(game);
            }
        }

        games.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(games)
    }

    /// `list()` filtered to games released for players — the query the
    /// front-end (via the gdext bridge) actually needs; `arcade list`
    /// itself still wants every installed game, released or not, so this
    /// doesn't replace `list()`.
    pub fn list_released(&self) -> io::Result<Vec<GameConfig>> {
        Ok(self.list()?.into_iter().filter(|g| g.released_for_players).collect())
    }

    /// Writes (creating or overwriting) `game`'s manifest. Used both for a
    /// fresh install and for `arcade update` refreshing an existing one —
    /// unlike the old single-file registry, there's no "already exists"
    /// error to raise here; that decision belongs to the caller, which can
    /// check `get()` first and decide whether that's a real conflict or a
    /// stale leftover safe to just overwrite.
    pub fn save(&self, game: &GameConfig) -> io::Result<()> {
        let dir = self.root.join(&game.name);
        fs::create_dir_all(&dir)?;
        let contents = serde_json::to_string_pretty(game).expect("GameConfig always serializes");
        fs::write(self.manifest_path(&game.name), contents)
    }

    /// Deletes a game's entire folder (manifest, preview image, and
    /// provisioned files together) — see the struct doc comment on why
    /// that's the whole removal, not two steps that could drift apart.
    pub fn remove(&self, name: &str) -> Result<(), StoreError> {
        if self.get(name).is_none() {
            return Err(StoreError::NotFound(name.to_string()));
        }
        fs::remove_dir_all(self.root.join(name))?;
        Ok(())
    }

    pub fn set_released(&self, name: &str, released: bool) -> Result<(), StoreError> {
        let mut game = self.get(name).ok_or_else(|| StoreError::NotFound(name.to_string()))?;
        game.released_for_players = released;
        self.save(&game)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum StoreError {
    NotFound(String),
    Io(io::Error),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound(name) => write!(f, "no game named '{name}' found"),
            StoreError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (GameStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = GameStore::open(dir.path());
        (store, dir)
    }

    fn sample(name: &str) -> GameConfig {
        GameConfig {
            name: name.to_string(),
            source: "steam".to_string(),
            source_ref: "70".to_string(),
            branch: None,
            exec: None,
            proton: false,
            prefix_path: None,
            released_for_players: false,
            description: None,
            tags: vec![],
            image_path: None,
        }
    }

    #[test]
    fn list_is_empty_before_anything_is_added() {
        let (store, _dir) = temp_store();
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn save_then_list_roundtrips() {
        let (store, _dir) = temp_store();
        store.save(&sample("half-life")).unwrap();
        assert_eq!(store.list().unwrap(), vec![sample("half-life")]);
    }

    #[test]
    fn list_released_only_returns_released_games() {
        let (store, _dir) = temp_store();
        store.save(&sample("hidden")).unwrap();
        let mut released = sample("released");
        released.released_for_players = true;
        store.save(&released).unwrap();

        assert_eq!(store.list_released().unwrap(), vec![released]);
    }

    #[test]
    fn image_abs_path_is_relative_to_the_games_own_folder() {
        let mut game = sample("half-life");
        game.image_path = Some("capsule.jpg".to_string());
        assert_eq!(game.image_abs_path(), Some(crate::game_dir("half-life").join("capsule.jpg")));
    }

    #[test]
    fn image_abs_path_is_none_without_an_image() {
        assert_eq!(sample("half-life").image_abs_path(), None);
    }

    #[test]
    fn save_twice_overwrites_rather_than_erroring() {
        let (store, _dir) = temp_store();
        store.save(&sample("half-life")).unwrap();
        let mut updated = sample("half-life");
        updated.source_ref = "220".to_string();
        store.save(&updated).unwrap();
        assert_eq!(store.list().unwrap(), vec![updated]);
    }

    #[test]
    fn a_folder_with_no_manifest_is_treated_as_not_installed() {
        let (store, dir) = temp_store();
        fs::create_dir_all(dir.path().join("half-life")).unwrap();
        assert_eq!(store.get("half-life"), None);
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn a_folder_with_a_corrupt_manifest_is_treated_as_not_installed() {
        let (store, dir) = temp_store();
        let game_dir = dir.path().join("half-life");
        fs::create_dir_all(&game_dir).unwrap();
        fs::write(game_dir.join("manifest.json"), "{not valid json").unwrap();
        assert_eq!(store.get("half-life"), None);
        assert_eq!(store.list().unwrap(), vec![]);
    }

    #[test]
    fn save_recovers_a_folder_with_a_corrupt_manifest() {
        let (store, dir) = temp_store();
        let game_dir = dir.path().join("half-life");
        fs::create_dir_all(&game_dir).unwrap();
        fs::write(game_dir.join("manifest.json"), "{not valid json").unwrap();

        store.save(&sample("half-life")).unwrap();
        assert_eq!(store.get("half-life"), Some(sample("half-life")));
    }

    #[test]
    fn remove_deletes_and_errors_if_missing() {
        let (store, _dir) = temp_store();
        store.save(&sample("half-life")).unwrap();
        store.remove("half-life").unwrap();
        assert_eq!(store.list().unwrap(), vec![]);
        assert!(matches!(
            store.remove("half-life").unwrap_err(),
            StoreError::NotFound(_)
        ));
    }

    #[test]
    fn set_released_toggles_the_flag() {
        let (store, _dir) = temp_store();
        store.save(&sample("half-life")).unwrap();
        store.set_released("half-life", true).unwrap();
        assert!(store.get("half-life").unwrap().released_for_players);
        store.set_released("half-life", false).unwrap();
        assert!(!store.get("half-life").unwrap().released_for_players);
    }

    #[test]
    fn persists_across_separate_store_instances() {
        let dir = tempfile::tempdir().unwrap();
        GameStore::open(dir.path()).save(&sample("half-life")).unwrap();
        let reopened = GameStore::open(dir.path());
        assert_eq!(reopened.list().unwrap(), vec![sample("half-life")]);
    }
}
